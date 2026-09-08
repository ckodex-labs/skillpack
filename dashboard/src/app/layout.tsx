import './ckodex-ds3-core.css';
import './ckodex-ds3-surfaces.css';
import './globals.css';
import './globals-panels.css';
/* Merged console (skill-registry) DS-3 surface — imported last so its
   canonical --ck-* tokens and ckr-* classes take precedence. */
import '../console/ds-styles.css';
import '../console/ds3-bridge.css';
import '../console/a11y-themes.css';
import '../console/console.css';

export const metadata = {
  title: 'Fable Skills Fleet — CKODEX Skill Gallery',
  description:
    'A governed fleet of skill hero pages. Each skill presented as a trust-bearing, evidence-native product surface.',
};

/* Restore the persisted theme before paint. Forced-colors is never
   gated: it activates from the OS via media query in the stylesheet. */
const THEME_RESTORE_SCRIPT = `try{var t=localStorage.getItem('ck-theme');if(t==='vault'||t==='hc'||t==='ledger'){document.documentElement.setAttribute('data-theme',t);}}catch(e){}`;

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  /* data-theme is set only by the restore script and the theme switch;
     declaring it in JSX would let React 19 hydration overwrite the
     restored value. Without the attribute, :root resolves to ledger. */
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link
          rel="preconnect"
          href="https://fonts.gstatic.com"
          crossOrigin="anonymous"
        />
        <meta name="theme-color" content="#F6F1E8" />
        <script dangerouslySetInnerHTML={{ __html: THEME_RESTORE_SCRIPT }} />
      </head>
      <body>
        <a href="#main-content" className="skip-link">
          Skip to main content
        </a>
        {children}
      </body>
    </html>
  );
}
