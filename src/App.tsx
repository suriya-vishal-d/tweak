import { ThemeProvider } from '@/store'
import { ThemeToggle, RequestBuilder } from '@/components'
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
          <div className={styles.content}>
            <RequestBuilder />
          </div>
        </main>
      </div>
    </ThemeProvider>
  )
}

export default App
