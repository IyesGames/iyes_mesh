use crate::HashMap;
use crate::descriptor::*;

#[derive(Default, Clone)]
pub struct MeshDataRef<'s> {
    pub indices: Option<(IndexFormat, &'s [u8])>,
    pub attributes: HashMap<VertexUsage, (VertexFormat, &'s [u8])>,
}

impl<'s> MeshDataRef<'s> {
    pub fn n_vertices(&self) -> usize {
        let Some(first) = self.attributes.values().next() else {
            return 0;
        };
        first.1.len() / first.0.size()
    }

    pub fn n_indices(&self) -> Option<usize> {
        self.indices.map(|b| b.1.len() / b.0.size())
    }

    pub fn validate(&self) -> bool {
        if self.attributes.is_empty() {
            return false;
        }
        let n_vertices = self.n_vertices();
        if let Some(b) = self.indices {
            let n_indices = b.1.len() / b.0.size();
            if !validate_buf(n_indices, b.0.size(), b.1) {
                return false;
            }
        }
        for b in self.attributes.values() {
            if !validate_buf(n_vertices, b.0.size(), b.1) {
                return false;
            }
        }
        true
    }
}

fn validate_buf(
    n_vertices: usize,
    fmt_size: usize,
    buf: &[u8],
) -> bool {
    buf.len() % fmt_size == 0 && buf.len() / fmt_size == n_vertices
}
