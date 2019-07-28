use graphql_parser::{query as q, query::Name, schema as s, schema::ObjectType};
use std::collections::{BTreeMap, HashMap};

use graph::data::graphql::values::{TryFromValue, ValueList, ValueMap};
use graph::data::subgraph::schema::SUBGRAPHS_ID;
use graph::prelude::*;
use graph_graphql::prelude::{object_value, ObjectOrInterface, Resolver};

pub struct IndexNodeResolver<R, S> {
    logger: Logger,
    graphql_runner: Arc<R>,
    store: Arc<S>,
}

struct IndexingStatus {
    id: String,
    synced: bool,
    failed: bool,
    error: Option<String>,
}

struct IndexingStatuses(Vec<IndexingStatus>);

impl From<&QueryResult> for IndexingStatuses {
    fn from(result: &QueryResult) -> Self {
        IndexingStatuses(result.data.map_or(vec![], |value| {
            let deployments = match value
                .get_required("subgraphDeployments")
                .expect("no subgraph deployments in the result")
            {
                q::Value::List(values) => values.get_values(),
                _ => vec![],
            };

            deployments.map(|deployment| IndexingStatus {
                id: deployment.get_required("id"),
                synced: deployment.get_required("synced"),
                failed: deployment.get_required("failed"),
                error: None,
            })
        }))
    }
}

impl<R, S> IndexNodeResolver<R, S>
where
    R: GraphQlRunner,
    S: Store + SubgraphDeploymentStore,
{
    pub fn new(logger: &Logger, graphql_runner: Arc<R>, store: Arc<S>) -> Self {
        let logger = logger.new(o!("component" => "IndexNodeResolver"));
        Self {
            logger,
            graphql_runner,
            store,
        }
    }

    fn resolve_indexing_statuses(
        &self,
        arguments: &HashMap<&q::Name, q::Value>,
    ) -> Result<q::Value, QueryExecutionError> {
        let schema = self
            .store
            .subgraph_schema(&SUBGRAPHS_ID)
            .map_err(QueryExecutionError::StoreError)?;

        let query = Query {
            schema,
            document: q::parse_query(
                r#"
                query deployments($where: SubgraphDeployment_filter!) {
                  subgraphDeployments(where: $where) {
                    id
                    synced
                    failed
                  }
                }
                "#,
            )
            .expect("invalid deployments query"),
            variables: arguments
                .get(&String::from("subgraphs"))
                .map(|value| match value {
                    ids @ q::Value::List(_) => QueryVariables::new(HashMap::from_iter(
                        vec![("where".into(), object_value(vec![("id_in", ids.clone())]))]
                            .into_iter(),
                    )),
                    _ => unreachable!(),
                }),
        };

        let result = self
            .graphql_runner
            .run_query_with_complexity(query, None)
            .wait()
            .expect("error querying subgraph deployments");

        let statuses = IndexingStatuses::from(&result);

        Ok(q::Value::List(vec![]))
    }
}

impl<R, S> Clone for IndexNodeResolver<R, S>
where
    R: GraphQlRunner,
    S: Store + SubgraphDeploymentStore,
{
    fn clone(&self) -> Self {
        Self {
            logger: self.logger.clone(),
            graphql_runner: self.graphql_runner.clone(),
            store: self.store.clone(),
        }
    }
}

impl<R, S> Resolver for IndexNodeResolver<R, S>
where
    R: GraphQlRunner,
    S: Store + SubgraphDeploymentStore,
{
    fn resolve_objects(
        &self,
        parent: &Option<q::Value>,
        field: &q::Name,
        field_definition: &s::Field,
        object_type: ObjectOrInterface<'_>,
        arguments: &HashMap<&q::Name, q::Value>,
        types_for_interface: &BTreeMap<Name, Vec<ObjectType>>,
    ) -> Result<q::Value, QueryExecutionError> {
        dbg!("Resolve objects");
        dbg!(field);
        dbg!(arguments);

        match (parent, field.as_str(), object_type.name()) {
            (None, "indexingStatuses", "SubgraphIndexingStatus") => {
                self.resolve_indexing_statuses(arguments)
            }
            (None, name, _) => Err(QueryExecutionError::UnknownField(
                field_definition.position.clone(),
                "Query".into(),
                name.into(),
            )),
            (_, name, _) => Err(QueryExecutionError::Unimplemented(format!(
                "Unknown field `{}`",
                name
            ))),
        }
    }

    fn resolve_object(
        &self,
        parent: &Option<q::Value>,
        field: &q::Field,
        field_definition: &s::Field,
        object_type: ObjectOrInterface<'_>,
        arguments: &HashMap<&q::Name, q::Value>,
        types_for_interface: &BTreeMap<Name, Vec<ObjectType>>,
    ) -> Result<q::Value, QueryExecutionError> {
        dbg!("Resolve object");
        dbg!(field);
        dbg!(object_type);
        dbg!(arguments);
        Ok(q::Value::Null)
    }
}
