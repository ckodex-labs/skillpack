import { defineConfig } from "rspress/config";

export default defineConfig({
  root: "docs",
  title: "SkillPack",
  description: "AI Agent Skill Quality Assessment Framework",
  logo: "/logo.svg",
  themeConfig: {
    socialLinks: [
      { icon: "github", link: "https://github.com/ckodex/skillpack" },
    ],
    sidebar: {
      "/guide/": [
        { text: "Getting Started", link: "/guide/getting-started" },
        { text: "Architecture", link: "/guide/architecture" },
        { text: "Dimensions", link: "/guide/dimensions" },
        { text: "Remote Sources", link: "/guide/remote-sources" },
        { text: "CLI Reference", link: "/guide/cli" },
        { text: "gRPC API", link: "/guide/grpc-api" },
        { text: "Security", link: "/guide/security" },
      ],
      "/specs/": [
        { text: "Skills Dossier", link: "/specs/skills-dossier" },
        { text: "RFC-001 Lifecycle", link: "/specs/rfc-001-lifecycle" },
        { text: "Skill Spec v1.1", link: "/specs/skill-spec-v1.1" },
        { text: "STX Spec", link: "/specs/stx-spec" },
        { text: "Client Spec", link: "/specs/client-spec" },
        { text: "Schema Index", link: "/specs/schema-index" },
      ],
      "/skills/": [
        {
          text: "Agentic Skill Template",
          link: "/skills/agentic-skill-template",
        },
      ],
    },
  },
});
