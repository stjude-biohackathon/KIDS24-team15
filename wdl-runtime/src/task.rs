//! Representation of task evaluation.

use std::{collections::HashMap, path::Path};

use indexmap::IndexMap;
use petgraph::{
    algo::{has_path_connecting, toposort},
    graph::{DiGraph, NodeIndex},
};
use wdl_ast::{
    v1::{
        CommandPart, CommandSection, Decl, HintsSection, NameRef, RequirementsSection,
        RuntimeSection, TaskDefinition, TaskItem,
    },
    AstNode, AstToken, Diagnostic, Ident, SyntaxNode, TokenStrHash,
};

use crate::{util::strip_leading_whitespace, v1::ExprEvaluator, Runtime, Value};

/// Creates a "missing input" diagnostic.
fn missing_input(task: &str, input: &Ident) -> Diagnostic {
    Diagnostic::error(format!(
        "missing input `{input}` for task `{task}`",
        input = input.as_str()
    ))
    .with_label("a value must be specified for this input", input.span())
}

/// Represents a node in an evaluation graph.
#[derive(Debug, Clone)]
pub enum GraphNode {
    /// The node is an input.
    Input(Decl),
    /// The node is a private decl.
    Decl(Decl),
    /// The node is an output decl.
    Output(Decl),
    /// The node is the task's command.
    Command(CommandSection),
    /// The node is a runtime section.
    Runtime(RuntimeSection),
    /// The node is a requirements section.
    Requirements(RequirementsSection),
    /// The node is a hints section.
    Hints(HintsSection),
}

/// Represents a task evaluation graph.
///
/// This is used to evaluate declarations and sections in topological order.
#[derive(Debug, Default)]
pub struct TaskEvaluationGraph {
    /// The inner directed graph.
    ///
    /// Note that edges in this graph are in *reverse* dependency ordering (implies "depended upon by" relationships).
    inner: DiGraph<GraphNode, ()>,
    /// The map of declaration names to node indexes in the graph.
    names: IndexMap<TokenStrHash<Ident>, NodeIndex>,
    /// The command node index.
    command: Option<NodeIndex>,
    /// The runtime node index.
    runtime: Option<NodeIndex>,
    /// The requirements node index.
    requirements: Option<NodeIndex>,
    /// The hints node index.
    hints: Option<NodeIndex>,
}

impl TaskEvaluationGraph {
    /// Constructs a new task evaluation graph.
    pub fn new(task: &TaskDefinition) -> Self {
        // Populate the declaration types and build a name reference graph
        let mut saw_inputs = false;
        let mut outputs = None;
        let mut graph = Self::default();
        for item in task.items() {
            match item {
                TaskItem::Input(section) if !saw_inputs => {
                    saw_inputs = true;
                    for decl in section.declarations() {
                        graph.add_decl_node(decl, GraphNode::Input);
                    }
                }
                TaskItem::Output(section) if outputs.is_none() => {
                    outputs = Some(section);
                }
                TaskItem::Declaration(decl) => {
                    graph.add_decl_node(Decl::Bound(decl), GraphNode::Decl);
                }
                TaskItem::Command(section) if graph.command.is_none() => {
                    graph.command = Some(graph.inner.add_node(GraphNode::Command(section)));
                }
                TaskItem::Runtime(section)
                    if graph.runtime.is_none() && graph.requirements.is_none() =>
                {
                    graph.runtime = Some(graph.inner.add_node(GraphNode::Runtime(section)));
                }
                TaskItem::Requirements(section)
                    if graph.requirements.is_none() && graph.runtime.is_none() =>
                {
                    graph.requirements =
                        Some(graph.inner.add_node(GraphNode::Requirements(section)));
                }
                TaskItem::Hints(section) if graph.hints.is_none() && graph.runtime.is_none() => {
                    graph.hints = Some(graph.inner.add_node(GraphNode::Hints(section)));
                }
                _ => continue,
            }
        }

        // Add name reference edges before adding the outputs
        graph.add_reference_edges(None);

        let count = graph.inner.node_count();

        if let Some(section) = outputs {
            for decl in section.declarations() {
                if let Some(index) = graph.add_decl_node(Decl::Bound(decl), GraphNode::Output) {
                    // Add an edge to the command node as all outputs depend on the command
                    if let Some(command) = graph.command {
                        graph.inner.update_edge(command, index, ());
                    }
                }
            }
        }

        // Add reference edges again, but only for the output declaration nodes
        graph.add_reference_edges(Some(count));

        // Finally, add edges from the command to runtime/requirements/hints
        if let Some(command) = graph.command {
            if let Some(runtime) = graph.runtime {
                graph.inner.update_edge(runtime, command, ());
            }

            if let Some(requirements) = graph.requirements {
                graph.inner.update_edge(requirements, command, ());
            }

            if let Some(hints) = graph.hints {
                graph.inner.update_edge(hints, command, ());
            }
        }

        graph
    }

    /// Performs a topological sort of the graph nodes.
    pub fn toposort(&self) -> Vec<GraphNode> {
        toposort(&self.inner, None)
            .expect("graph should be acyclic")
            .into_iter()
            .map(|i| self.inner[i].clone())
            .collect()
    }

    /// Adds a declaration node to the graph.
    fn add_decl_node<F>(&mut self, decl: Decl, map: F) -> Option<NodeIndex>
    where
        F: FnOnce(Decl) -> GraphNode,
    {
        let name = decl.name();

        // Ignore duplicate nodes
        if self.names.contains_key(name.as_str()) {
            return None;
        }

        let index = self.inner.add_node(map(decl));
        self.names.insert(TokenStrHash::new(name), index);
        Some(index)
    }

    /// Adds edges from task sections to declarations.
    fn add_section_edges(
        &mut self,
        from: NodeIndex,
        descendants: impl Iterator<Item = SyntaxNode>,
    ) {
        // Add edges for any descendant name references
        for r in descendants.filter_map(NameRef::cast) {
            let name = r.name();

            // Look up the name; we don't check for cycles here as decls can't
            // reference a section.
            if let Some(to) = self.names.get(name.as_str()) {
                self.inner.update_edge(*to, from, ());
            }
        }
    }

    /// Adds name reference edges to the graph.
    fn add_reference_edges(&mut self, skip: Option<usize>) {
        let mut space = Default::default();

        // Populate edges for any nodes that reference other nodes by name
        for from in self.inner.node_indices().skip(skip.unwrap_or(0)) {
            match &self.inner[from] {
                GraphNode::Input(decl) | GraphNode::Decl(decl) | GraphNode::Output(decl) => {
                    let expr = decl.expr();
                    if let Some(expr) = expr {
                        for r in expr.syntax().descendants().filter_map(NameRef::cast) {
                            let name = r.name();

                            // Only add an edge if the name is known to us
                            if let Some(to) = self.names.get(name.as_str()) {
                                // Ignore edges that form cycles; evaluation will later treat this as an unknown name reference.
                                if has_path_connecting(&self.inner, from, *to, Some(&mut space)) {
                                    continue;
                                }

                                self.inner.update_edge(*to, from, ());
                            }
                        }
                    }
                }
                GraphNode::Command(section) => {
                    // Add name references from the command section to any decls in scope
                    let section = section.clone();
                    for part in section.parts() {
                        if let CommandPart::Placeholder(p) = part {
                            self.add_section_edges(from, p.syntax().descendants());
                        }
                    }
                }
                GraphNode::Runtime(section) => {
                    // Add name references from the runtime section to any decls in scope
                    let section = section.clone();
                    for item in section.items() {
                        self.add_section_edges(from, item.syntax().descendants());
                    }
                }
                GraphNode::Requirements(section) => {
                    // Add name references from the requirements section to any decls in scope
                    let section = section.clone();
                    for item in section.items() {
                        self.add_section_edges(from, item.syntax().descendants());
                    }
                }
                GraphNode::Hints(section) => {
                    // Add name references from the hints section to any decls in scope
                    let section = section.clone();
                    for item in section.items() {
                        self.add_section_edges(from, item.syntax().descendants());
                    }
                }
            }
        }
    }
}

/// Represents an evaluated task.
#[derive(Debug)]
pub struct EvaluatedTask<'a> {
    /// The evaluated command text (i.e. bash script) to use for executing the task.
    command: String,
    /// The evaluated requirements for running the command.
    requirements: IndexMap<String, Value>,
    /// The evaluated hints for running the command.
    hints: IndexMap<String, Value>,
    /// The map from input paths to localized paths within the execution environment.
    paths: IndexMap<String, String>,
    /// The evaluation scope for evaluating the task so far.
    scope: HashMap<TokenStrHash<Ident>, Value>,
    /// The evaluation nodes; this is used to evaluate the outputs after the task is executed.
    nodes: &'a Vec<GraphNode>,
    /// The index of the start of the outputs in the set of nodes.
    outputs: Option<usize>,
}

impl<'a> EvaluatedTask<'a> {
    /// Constructs a new evaluated task given the graph nodes in topological order.
    fn new(nodes: &'a Vec<GraphNode>) -> Self {
        Self {
            command: String::new(),
            requirements: Default::default(),
            hints: Default::default(),
            paths: Default::default(),
            scope: Default::default(),
            nodes,
            outputs: None,
        }
    }

    /// Gets the command (i.e. bash script) to use for executing the task.
    pub fn command(&self) -> &str {
        &self.command
    }

    /// The localized paths used by the command.
    ///
    /// The key is the local path and the value is the localized path.
    pub fn paths(&self) -> &IndexMap<String, String> {
        &self.paths
    }

    /// The evaluated requirements for running the command.
    pub fn requirements(&self) -> &IndexMap<String, Value> {
        &self.requirements
    }

    /// The evaluated hints for running the command.
    pub fn hints(&self) -> &IndexMap<String, Value> {
        &self.hints
    }

    /// Evaluates the outputs of the task given the stdout and stderr from task execution.
    pub fn outputs(
        &self,
        runtime: &mut Runtime<'_>,
        stdout: impl AsRef<Path>,
        stderr: impl AsRef<Path>,
    ) -> Result<HashMap<TokenStrHash<Ident>, Value>, Diagnostic> {
        let mut outputs = HashMap::default();
        if let Some(index) = self.outputs {
            let stdout = runtime.new_file(stdout.as_ref().to_string_lossy());
            let stderr = runtime.new_file(stderr.as_ref().to_string_lossy());

            let evaluator = ExprEvaluator::new_with_output(&self.scope, stdout, stderr);
            for node in &self.nodes[index..] {
                match node {
                    GraphNode::Output(decl) => {
                        let name = decl.name();
                        let expr = decl.expr().expect("decl should be bound");
                        let value = evaluator.evaluate_expr(runtime, &expr)?;
                        outputs.insert(TokenStrHash::new(name), value);
                    }
                    _ => panic!("only output nodes should follow the command"),
                }
            }
        }

        Ok(outputs)
    }
}

/// Represents a task evaluator.
#[derive(Debug)]
pub struct TaskEvaluator {
    /// The name of the task being evaluated.
    name: Ident,
    /// The task evaluation nodes in topological order.
    nodes: Vec<GraphNode>,
}

impl TaskEvaluator {
    /// Constructs a new task based on a definition and its inputs.
    pub fn new(definition: TaskDefinition) -> Self {
        let graph = TaskEvaluationGraph::new(&definition);
        let nodes = graph.toposort();
        Self {
            name: definition.name(),
            nodes,
        }
    }

    /// Evaluates the task with the given base path to use for file localization.
    pub fn evaluate<'a>(
        &'a self,
        runtime: &mut Runtime<'_>,
        inputs: &HashMap<String, Value>,
        _base: impl AsRef<Path>,
    ) -> Result<EvaluatedTask<'a>, Diagnostic> {
        let mut evaluated = EvaluatedTask::new(&self.nodes);

        // Start by walking the nodes looking for input decls to populate the scope
        for node in &self.nodes {
            match node {
                GraphNode::Input(decl) => {
                    let name = decl.name();
                    if let Some(value) = inputs.get(name.as_str()) {
                        evaluated.scope.insert(TokenStrHash::new(name), *value);
                    } else {
                        // Check to see if the declaration was unbound; if so, it may be required if the declared type is not optional
                        if let Decl::Unbound(decl) = decl {
                            let ty = decl.ty();
                            if ty.is_optional() {
                                evaluated.scope.insert(TokenStrHash::new(name), Value::None);
                            } else {
                                // The input is required
                                return Err(missing_input(self.name.as_str(), &name));
                            }
                        }
                    }
                }
                GraphNode::Decl(_) => continue,
                _ => break,
            }
        }

        // Walk the nodes again and evaluate them
        for (index, node) in self.nodes.iter().enumerate() {
            match node {
                GraphNode::Input(decl) | GraphNode::Decl(decl) => {
                    let name = decl.name();
                    if evaluated.scope.contains_key(name.as_str()) {
                        // Skip evaluating the input as we already have the value in scope
                        continue;
                    }

                    let expr = decl.expr().expect("declaration should be bound");
                    let evaluator = ExprEvaluator::new(&evaluated.scope);
                    let value = evaluator.evaluate_expr(runtime, &expr)?;
                    evaluated.scope.insert(TokenStrHash::new(name), value);
                }
                GraphNode::Requirements(_) => {
                    // TODO: implement
                }
                GraphNode::Runtime(_) => {
                    // TODO: implement
                }
                GraphNode::Hints(_) => {
                    // TODO: implement
                }
                GraphNode::Command(section) => {
                    // TODO: set `task` variable in scope for 1.2 documents
                    let evaluator = ExprEvaluator::new(&evaluated.scope);
                    for part in section.parts() {
                        match part {
                            CommandPart::Text(text) => evaluated.command.push_str(text.as_str()),
                            CommandPart::Placeholder(placeholder) => evaluator
                                .evaluate_placeholder(
                                    runtime,
                                    &placeholder,
                                    &mut evaluated.command,
                                )?,
                        }
                    }

                    evaluated.command = strip_leading_whitespace(&evaluated.command, true);
                }
                GraphNode::Output(_) => {
                    evaluated.outputs = Some(index);
                    break;
                }
            }
        }

        Ok(evaluated)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    use tempfile::TempDir;
    use wdl_analysis::Analyzer;

    #[tokio::test]
    async fn it_works() {
        let dir = TempDir::new().expect("failed to create temporary directory");
        let path = dir.path().join("foo.wdl");
        fs::write(
            &path,
            r#"version 1.1

task test {
    input {
        String name = "Peter"
    }
    
    command <<<
        echo Hi, ~{name}!
    >>>

    output {
        String message = "stdout was: ~{read_string(stdout())}"
    }
}
"#,
        )
        .expect("failed to create test file");

        let stdout = dir.path().join("stdout");
        fs::write(&stdout, r#"Hi, Peter!"#).expect("failed to create test file");

        let stderr = dir.path().join("stderr");
        fs::write(&stderr, r#""#).expect("failed to create test file");

        let analyzer = Analyzer::new(|_: (), _, _, _| async {});
        analyzer
            .add_documents(vec![dir.path().to_path_buf()])
            .await
            .expect("should add documents");

        let results = analyzer.analyze(()).await.expect("should succeed");
        assert_eq!(results.len(), 1);
        assert!(results[0].diagnostics().is_empty());

        let document = results[0]
            .parse_result()
            .document()
            .expect("should have a document");

        let task = document
            .ast()
            .as_v1()
            .expect("should be a V1 AST")
            .tasks()
            .find(|t| t.name().as_str() == "test")
            .expect("should have task");

        let mut runtime = Runtime::new(results[0].scope());
        let inputs = HashMap::new();
        let evaluator = TaskEvaluator::new(task);
        let evaluated = evaluator
            .evaluate(&mut runtime, &inputs, "/tmp")
            .expect("should evaluate");

        let outputs = evaluated
            .outputs(&mut runtime, stdout, stderr)
            .expect("should evaluate");
        for (k, v) in outputs {
            assert_eq!(k.as_ref().as_str(), "message");
            match v {
                Value::String(sym) => {
                    assert_eq!(runtime.resolve_str(sym), "stdout was: Hi, Peter!")
                }
                _ => panic!("expected a string value, found {v:?}"),
            }
        }
    }
}
