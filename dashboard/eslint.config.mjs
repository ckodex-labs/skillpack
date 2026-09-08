import nextConfig from 'eslint-config-next';

/* Next.js 16 removed `next lint`; ESLint runs directly against the
   flat config that eslint-config-next now ships. The react version is
   pinned because eslint-plugin-react's auto-detection relies on an
   ESLint 9 API that ESLint 10 removed. */
export default [
  ...nextConfig,
  {
    settings: {
      react: { version: '19.2' },
    },
  },
  {
    ignores: ['.next/**', 'node_modules/**', 'src/generated/**'],
  },
];
