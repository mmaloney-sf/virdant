use std::sync::Arc;

/// A [`Context`] is an associative list which assigns each element with type information.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Context<K, T>(Arc<Vec<(K, T)>>);

impl<K: Eq + Clone, T: Clone> Context<K, T> {
    pub fn empty() -> Context<K, T> {
        Context(Arc::new(vec![]))
    }

    pub fn from(ctx: Vec<(K, T)>) -> Context<K, T> {
        Context(Arc::new(ctx))
    }

    pub fn lookup(&self, v: &K) -> Option<T> {
        for (v0, t) in self.0.iter().rev() {
            if v0 == v {
                return Some(t.clone());
            }
        }
        None
    }

    pub fn extend(&self, v: K, t: T) -> Context<K, T> {
        let mut result = self.to_vec();
        result.push((v, t));
        Context(Arc::new(result))
    }

    pub fn extend_from(&self, another: &Context<K, T>) -> Context<K, T> {
        let mut result = self.to_vec();
        for (v, t) in another.0.iter() {
            result.push((v.clone(), t.clone()));
        }
        Context(Arc::new(result))
    }

    pub fn into_inner(self) -> Vec<(K, T)> {
        self.0.to_vec()
    }
}

impl<K, T> std::ops::Deref for Context<K, T> {
    type Target = Vec<(K, T)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K: std::fmt::Display, T: std::fmt::Display> std::fmt::Display for Context<K, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, (name, v)) in self.0.iter().enumerate() {
            write!(f, "{name} : {v}")?;
            if i + 1 < self.0.len() {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}
