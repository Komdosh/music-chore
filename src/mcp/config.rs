//! Configuration management for Music Chore MCP server
//!
//! This module handles environment variable configuration and validation.

use std::env;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    /// Logging level (error, warn, info, debug, trace)
    pub log_level: String,
    /// Default music library path
    pub default_library_path: Option<PathBuf>,
    /// Scan timeout in seconds
    pub scan_timeout: Duration,
    /// Allowed paths for security
    pub allowed_paths: Vec<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            default_library_path: None,
            scan_timeout: Duration::from_secs(300),
            allowed_paths: vec![],
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Logging level
        if let Ok(log_level) = env::var("RUST_LOG") {
            config.log_level = log_level;
        }

        // Default music library path
        if let Ok(path) = env::var("MUSIC_LIBRARY_PATH") {
            config.default_library_path = Some(PathBuf::from(path));
        }

        // Scan timeout
        if let Ok(timeout_str) = env::var("MUSIC_SCAN_TIMEOUT") {
            if let Ok(timeout_secs) = timeout_str.parse::<u64>() {
                config.scan_timeout = Duration::from_secs(timeout_secs);
            }
        }

        // Allowed paths for security
        if let Ok(allowed_paths_str) = env::var("MUSIC_ALLOWED_PATHS") {
            config.allowed_paths = allowed_paths_str
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .map(|s| PathBuf::from(s.trim()))
                .collect();
        }

        config
    }

    /// Validate that a path is allowed based on security configuration
    pub fn is_path_allowed(&self, path: &PathBuf) -> bool {
        // If no allowed paths are configured, allow everything (backwards compatibility)
        if self.allowed_paths.is_empty() {
            return true;
        }

        // Check if the path is under any of the allowed paths
        self.allowed_paths
            .iter()
            .any(|allowed| path.starts_with(allowed) || path == allowed)
    }

    /// Get the default library path or return an error if none is set
    pub fn require_default_library_path(&self) -> Result<&PathBuf, String> {
        self.default_library_path
            .as_ref()
            .ok_or_else(|| "MUSIC_LIBRARY_PATH environment variable is not set".to_string())
    }

    /// Initialize logging based on configuration
    pub fn init_logging(&self) {
        let filter = match self.log_level.to_lowercase().as_str() {
            "error" => "error,lofty::flac::read=error",
            "warn" => "warn,lofty::flac::read=error",
            "info" => "info,lofty::flac::read=error",
            "debug" => "debug,lofty::flac::read=error",
            "trace" => "trace,lofty::flac::read=error",
            _ => "info,lofty::flac::read=error",
        };

        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(filter)).init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.log_level, "info");
        assert!(config.default_library_path.is_none());
        assert_eq!(config.scan_timeout, Duration::from_secs(300));
        assert!(config.allowed_paths.is_empty());
    }

    #[test]
    fn test_config_from_env() {
        // Set environment variables for testing
        unsafe {
            env::set_var("RUST_LOG", "debug");
            env::set_var("MUSIC_LIBRARY_PATH", "/test/music");
            env::set_var("MUSIC_SCAN_TIMEOUT", "600");
            env::set_var("MUSIC_ALLOWED_PATHS", "/music,/backup/music");
        }

        let config = Config::from_env();
        assert_eq!(config.log_level, "debug");
        assert_eq!(
            config.default_library_path,
            Some(PathBuf::from("/test/music"))
        );
        assert_eq!(config.scan_timeout, Duration::from_secs(600));
        assert_eq!(
            config.allowed_paths,
            vec![PathBuf::from("/music"), PathBuf::from("/backup/music")]
        );

        // Clean up
        unsafe {
            env::remove_var("RUST_LOG");
            env::remove_var("MUSIC_LIBRARY_PATH");
            env::remove_var("MUSIC_SCAN_TIMEOUT");
            env::remove_var("MUSIC_ALLOWED_PATHS");
        }
    }

    #[test]
    fn test_path_validation() {
        let mut config = Config::default();

        // With no allowed paths, everything is allowed
        assert!(config.is_path_allowed(&PathBuf::from("/any/path")));

        // With allowed paths configured
        config.allowed_paths = vec![PathBuf::from("/music"), PathBuf::from("/backup/music")];

        assert!(config.is_path_allowed(&PathBuf::from("/music/artist/album")));
        assert!(config.is_path_allowed(&PathBuf::from("/music")));
        assert!(config.is_path_allowed(&PathBuf::from("/backup/music/artist")));

        assert!(!config.is_path_allowed(&PathBuf::from("/other/path")));
        // Note: Path traversal detection requires more sophisticated validation
        // This simple prefix check focuses on basic security
    }

    #[test]
    fn test_require_default_library_path() {
        let mut config = Config::default();
        assert!(config.require_default_library_path().is_err());

        config.default_library_path = Some(PathBuf::from("/music"));
        assert_eq!(
            config.require_default_library_path().unwrap(),
            &PathBuf::from("/music")
        );
    }

    #[test]
    fn test_logging_initialization() {
        let config = Config {
            log_level: "debug".to_string(),
            ..Default::default()
        };

        // This test just ensures the function doesn't panic
        config.init_logging();
    }
}
