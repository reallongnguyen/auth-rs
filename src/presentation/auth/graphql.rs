mod mutation;
mod query;

use crate::context::Context;
use juniper::{EmptySubscription, RootNode};
use mutation::MutationRoot;
use query::QueryRoot;

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<Context>>;

pub fn create_schema() -> Schema {
  Schema::new(QueryRoot, MutationRoot, EmptySubscription::new())
}
