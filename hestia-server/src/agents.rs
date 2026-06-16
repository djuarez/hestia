/// A registered Hestia agent (one per Mac Mini).
#[derive(Debug, Clone)]
pub struct Agent {
    pub name: String,
    pub base_url: String,
}

/// In-memory registry of agents. v0.1 is populated statically from config;
/// dynamic registration can replace this later behind the same interface.
#[derive(Debug, Default)]
pub struct Registry {
    agents: Vec<Agent>,
}

impl Registry {
    /// Build a registry from specs of the form `name=url` or bare `url`.
    pub fn from_specs(specs: &[String]) -> Self {
        let agents = specs.iter().filter_map(|s| Agent::parse(s)).collect();
        Self { agents }
    }

    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    /// Look up an agent by its registered name.
    pub fn get(&self, name: &str) -> Option<&Agent> {
        self.agents.iter().find(|a| a.name == name)
    }

    pub fn len(&self) -> usize {
        self.agents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }
}

impl Agent {
    /// Parse one `name=url` (or bare `url`) spec. Returns `None` for blanks.
    fn parse(spec: &str) -> Option<Self> {
        let spec = spec.trim();
        if spec.is_empty() {
            return None;
        }

        let (name, url) = match spec.split_once('=') {
            Some((name, url)) => (name.trim().to_string(), url.trim()),
            None => (derive_name(spec), spec),
        };

        Some(Agent {
            name,
            base_url: url.trim_end_matches('/').to_string(),
        })
    }
}

/// Derive a display name from a URL: `http://10.0.0.11:4400/` -> `10.0.0.11:4400`.
fn derive_name(url: &str) -> String {
    let after_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    after_scheme
        .split('/')
        .next()
        .unwrap_or(after_scheme)
        .to_string()
}
