/** @type {import('next').NextConfig} */
const nextConfig = {
  // Pin the workspace root: a stray lockfile in a parent dir would otherwise
  // make Turbopack infer the wrong root and miss file changes (breaking HMR).
  turbopack: {
    root: __dirname,
  },
  experimental: {
    // Enable for gRPC-Web and Connect-ES
    serverActions: {
      bodySizeLimit: '2mb',
    },
  },
  env: {
    SKILLPACK_API_URL: process.env.SKILLPACK_API_URL || 'http://localhost:50051',
  },
}

module.exports = nextConfig
