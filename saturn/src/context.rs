use cedar_policy::{
    Authorizer, Context, Decision, Diagnostics, HumanSchemaError, ParseErrors, PolicySet,
    PolicySetError, Request, Schema, SchemaError, ValidationMode, Validator,
};
use itertools::Itertools;
use std::path::PathBuf;
use thiserror::Error;

use crate::{entitystore::EntityStore, util::EntityUid};

#[allow(dead_code)]
pub struct AppContext {
    entities: EntityStore,
    authorizer: Authorizer,
    policies: PolicySet,
    schema: Schema,
}

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("Error Parsing Json Schema: {0}")]
    JsonSchema(#[from] SchemaError),
    #[error("Error Parsing Human-readable Schema: {0}")]
    CedarSchema(#[from] HumanSchemaError),
    #[error("Error Parsing PolicySet: {0}")]
    Policy(#[from] ParseErrors),
    #[error("Error Processing PolicySet: {0}")]
    PolicySet(#[from] PolicySetError),
    #[error("Validation Failed: {0}")]
    Validation(String),
    #[error("Error Deserializing Json: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Authorization Denied")]
    AuthDenied(Diagnostics),
    #[error("Error constructing authorization request: {0}")]
    Request(String),
}

impl AppContext {
    pub fn new(
        entities_path: impl Into<PathBuf>,
        schema_path: impl Into<PathBuf>,
        policies_path: impl Into<PathBuf>,
    ) -> Result<Self, ContextError> {
        let schema_path = schema_path.into();
        let policies_path = policies_path.into();

        let schema_file = std::fs::File::open(schema_path)?;
        let (schema, _) = Schema::from_file_natural(schema_file).unwrap();
        let entities_file = std::fs::File::open(entities_path.into())?;
        let entities = serde_json::from_reader(entities_file)?;
        let policy_src = std::fs::read_to_string(policies_path)?;
        let policies = policy_src.parse()?;
        let validator = Validator::new(schema.clone());
        let output = validator.validate(&policies, ValidationMode::default());

        if output.validation_passed() {
            tracing::info!("All policy validation passed!");
            let authorizer = Authorizer::new();
            let c = Self {
                entities,
                authorizer,
                policies,
                schema,
            };

            Ok(c)
        } else {
            let error_string = output
                .validation_errors()
                .map(|err| format!("{err}"))
                .join("\n");
            Err(ContextError::Validation(error_string))
        }
    }

    pub fn is_authorized(
        &self,
        principal: impl AsRef<EntityUid>,
        action: impl AsRef<EntityUid>,
        resource: impl AsRef<EntityUid>,
        context: Context,
    ) -> Result<(), Error> {
        let es = self.entities.as_entities(&self.schema);
        let q = Request::new(
            Some(principal.as_ref().clone().into()),
            Some(action.as_ref().clone().into()),
            Some(resource.as_ref().clone().into()),
            context,
            Some(&self.schema),
        )
        .map_err(|e| Error::Request(e.to_string()))?;
        tracing::info!(
            "is_authorized request: principal: {}, action: {}, resource: {}",
            principal.as_ref(),
            action.as_ref(),
            resource.as_ref()
        );
        let response = self.authorizer.is_authorized(&q, &self.policies, &es);
        tracing::info!("Auth response: {:?}", response);
        match response.decision() {
            Decision::Allow => Ok(()),
            Decision::Deny => Err(Error::AuthDenied(response.diagnostics().clone())),
        }
    }
}
