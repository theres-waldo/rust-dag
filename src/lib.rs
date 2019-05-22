use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

pub struct Node<T> {
    pub data: T,
    pub incoming: Vec<NodeRef<T>>,
    pub outgoing: Vec<NodeRef<T>>,
}
impl<T> Node<T> {
    fn new(x: T) -> Self {
        Self {
            data: x,
            incoming: Vec::new(),
            outgoing: Vec::new(),
        }
    }
}

pub struct NodeRef<T> {
    pub ptr: Rc<RefCell<Node<T>>>,
}
impl<T> NodeRef<T> {
    fn new(data: T) -> NodeRef<T> {
        NodeRef {
            ptr: Rc::new(RefCell::new(Node::new(data))),
        }
    }
}
impl<T> Clone for NodeRef<T> {
    fn clone(&self) -> NodeRef<T> {
        NodeRef {
            ptr: self.ptr.clone(),
        }
    }
}

// Reference equality semantics for NodeRef<T>.
impl<T> Eq for NodeRef<T> {}
impl<T> PartialEq for NodeRef<T> {
    fn eq(&self, rhs: &NodeRef<T>) -> bool {
        self.ptr.as_ptr().eq(&rhs.ptr.as_ptr())
    }
}
impl<T> Hash for NodeRef<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.as_ptr().hash(state)
    }
}

fn remove_item<T>(vec: &mut Vec<T>, item: &T)
where
    T: PartialEq,
{
    vec.retain(|e| e != item)
}

#[derive(Default)]
pub struct DirectedGraph<T> {
    nodes: Vec<NodeRef<T>>,
}
impl<T> DirectedGraph<T> {
    pub fn add_node(&mut self, data: T) -> NodeRef<T> {
        let result = NodeRef::new(data);
        self.nodes.push(result.clone());
        result
    }

    pub fn add_edge(&mut self, from: &NodeRef<T>, to: &NodeRef<T>) {
        from.ptr.borrow_mut().outgoing.push(to.clone());
        to.ptr.borrow_mut().incoming.push(from.clone());
    }

    // Try to compute a topological sort using Kahn's algorithm.
    // This consumes the graph, because Kahn's algorithm involves removing incoming edges
    // as you go, but with a bit more effort we could write a version that preserves the graph.
    // If a topological sort exists, one is returned, otherwise None is returned.
    pub fn topological_sort(self) -> Option<Vec<NodeRef<T>>> {
        // result will contain the sorted elements
        let mut result = Vec::new();
        // S is a set of all nodes with no incoming edges
        let mut s: HashSet<_> = self
            .nodes
            .iter()
            .filter(|n| n.ptr.borrow().incoming.is_empty())
            .map(|e| e.clone())
            .collect();
        while !s.is_empty() {
            // remove a node n from S
            let n = s.iter().next().unwrap().clone();
            s.remove(&n);

            // add n to the tail of result
            result.push(n.clone());

            // for each node m with an edge e from n to m
            for m in &n.ptr.borrow().outgoing {
                // remove the edge e from the graph
                // (we only bother removing the incoming edge since that's all we need
                // and we are consuming the graph anyways)
                remove_item(&mut m.ptr.borrow_mut().incoming, &n);
                // if m has no other incoming edges
                if m.ptr.borrow().incoming.is_empty() {
                    // insert m into S
                    s.insert(m.clone());
                }
            }
        }
        // if the graph has remaining (incoming) edges, there is at least one cycle
        for node in &self.nodes {
            if !node.ptr.borrow().incoming.is_empty() {
                return None;
            }
        }
        // otherwise we have a topological sort
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_topological_sort() {
        // Create a DAG that looks like this:
        //      A
        //    /  \
        //   B    C
        //    \  /
        //     D
        let mut graph = DirectedGraph::default();
        let a = graph.add_node('A');
        let b = graph.add_node('B');
        let c = graph.add_node('C');
        let d = graph.add_node('D');
        graph.add_edge(&a, &b);
        graph.add_edge(&a, &c);
        graph.add_edge(&b, &d);
        graph.add_edge(&c, &d);

        // Compute a topological sort and check that it's correct.
        assert!(match graph.topological_sort() {
            None => false,
            Some(nodes) => {
                let str = nodes.iter().fold(String::new(), |acc, node| {
                    format!("{}{}", acc, node.ptr.borrow().data)
                });
                // These are the two possible topological sorts:
                str == "ABCD" || str == "ACBD"
            }
        })
    }
}
