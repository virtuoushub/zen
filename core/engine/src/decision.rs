use crate::engine::EvaluationOptions;
use crate::handler::tree::{GraphResponse, GraphTree, GraphTreeConfig};
use crate::loader::noop::NoopLoader;
use crate::loader::DecisionLoader;
use crate::model::decision::DecisionContent;
use anyhow::Context;
use serde_json::Value;
use std::sync::Arc;

pub struct Decision<L>
where
    L: DecisionLoader,
{
    content: Arc<DecisionContent>,
    loader: Arc<L>,
}

impl From<DecisionContent> for Decision<NoopLoader> {
    fn from(value: DecisionContent) -> Self {
        Self {
            content: value.into(),
            loader: NoopLoader::default().into(),
        }
    }
}

impl From<Arc<DecisionContent>> for Decision<NoopLoader> {
    fn from(value: Arc<DecisionContent>) -> Self {
        Self {
            content: value,
            loader: NoopLoader::default().into(),
        }
    }
}

impl<L> Decision<L>
where
    L: DecisionLoader,
{
    pub fn with_loader<NL>(self, loader: Arc<NL>) -> Decision<NL>
    where
        NL: DecisionLoader,
    {
        Decision {
            loader,
            content: self.content,
        }
    }

    pub async fn evaluate(&self, context: &Value) -> Result<GraphResponse, anyhow::Error> {
        self.evaluate_with_opts(context, Default::default()).await
    }

    pub async fn evaluate_with_opts(
        &self,
        context: &Value,
        options: EvaluationOptions,
    ) -> Result<GraphResponse, anyhow::Error> {
        let tree = GraphTree::new(GraphTreeConfig {
            max_depth: options.max_depth.unwrap_or(5),
            trace: options.trace.unwrap_or_default(),
            loader: self.loader.clone(),
            iteration: 0,
            content: &self.content,
        });

        tree.connect()?;
        tree.evaluate(context)
            .await
            .context("Failed to revaluate graph")
    }
}