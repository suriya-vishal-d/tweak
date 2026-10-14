import { useTheme } from '@/store'
import type { ThemeSetting } from '@/types'
import styles from './ThemeToggle.module.css'

const ICONS: Record<ThemeSetting, string> = {
  light: '☀️',
  dark: '🌙',
  system: '💻',
}

const LABELS: Record<ThemeSetting, string> = {
  light: 'Light theme',
  dark: 'Dark theme',
  system: 'System theme',
}

/**
 * ThemeToggle — cycles through light → dark → system on click.
 */
export function ThemeToggle() {
  const { setting, cycleTheme } = useTheme()

  return (
    <button
      className={styles.toggle}
      onClick={cycleTheme}
      aria-label={`Current theme: ${LABELS[setting]}. Click to cycle.`}
      title={LABELS[setting]}
    >
      <span className={styles.icon} aria-hidden="true">
        {ICONS[setting]}
      </span>
      <span className={styles.label}>{LABELS[setting]}</span>
    </button>
  )
}
