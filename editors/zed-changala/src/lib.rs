use zed_extension_api::{self as zed, Command, ContextServerId, Project};

struct ChangalaExtension;

impl zed::Extension for ChangalaExtension {
    fn new() -> Self {
        ChangalaExtension
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> zed::Result<Command> {
        // Changala MCP is a remote server — connection is configured via
        // settings.json with a "url" and "headers" entry, not a local command.
        //
        // This method is only called for local (stdio) MCP servers.
        // For remote servers, Zed connects directly to the configured URL.
        //
        // If the user hasn't configured the URL in settings, surface a
        // helpful error message.
        Err(
            "Changala MCP is a remote server. Configure it in your settings:\n\n\
             \"context_servers\": {\n  \
               \"changala\": {\n    \
                 \"url\": \"https://your-ring.example.com/mcp\",\n    \
                 \"headers\": {\n      \
                   \"Authorization\": \"Bearer <your-mcp-access-key>\"\n    \
                 }\n  \
               }\n\
             }"
            .into(),
        )
    }
}

zed::register_extension!(ChangalaExtension);
