import { ThemeProvider } from '@/store'
import { ThemeToggle } from '@/components'
import styles from './App.module.css'

function App() {
  return (
    <ThemeProvider>
      <div className={styles.app}>
        <header className={styles.header}>
          <div className={styles.headerContent}>
            <div className={styles.brand}>
              <span className={styles.brandIcon}>⚡</span>
              <h1 className={styles.brandName}>Tweak</h1>
              <span className={styles.brandTagline}>API Client</span>
            </div>
            <ThemeToggle />
          </div>
        </header>

        <main className={styles.main}>
          <div className={styles.hero}>
            <h2 className={styles.heroTitle}>Welcome to Tweak</h2>
            <p className={styles.heroSubtitle}>
              A lightweight, offline API client for Linux.
            </p>
            <p className={styles.heroNote}>
              🚧 <strong>Milestone M0 complete.</strong> Request builder coming in M1.
            </p>
          </div>
        </main>
      </div>
    </ThemeProvider>
  )
}

export default App

