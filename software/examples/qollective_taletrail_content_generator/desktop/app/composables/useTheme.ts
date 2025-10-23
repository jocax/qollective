import { ref, watch, onMounted } from 'vue'

export type ThemeMode = 'light' | 'dark' | 'system'

export function useTheme() {
  const theme = ref<ThemeMode>('system')
  const isDark = ref(false)

  function applyTheme(newTheme: ThemeMode) {
    theme.value = newTheme

    if (newTheme === 'dark') {
      isDark.value = true
      document.documentElement.classList.add('dark')
    } else if (newTheme === 'light') {
      isDark.value = false
      document.documentElement.classList.remove('dark')
    } else {
      // System preference
      const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches
      isDark.value = systemDark
      document.documentElement.classList.toggle('dark', systemDark)
    }

    // Save to localStorage
    if (typeof window !== 'undefined') {
      localStorage.setItem('theme', newTheme)
    }
  }

  function loadTheme() {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('theme') as ThemeMode
      applyTheme(saved || 'system')
    }
  }

  // Watch system preference changes
  onMounted(() => {
    loadTheme()

    // Listen for system theme changes
    if (typeof window !== 'undefined') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      const handleChange = (e: MediaQueryListEvent) => {
        if (theme.value === 'system') {
          isDark.value = e.matches
          document.documentElement.classList.toggle('dark', e.matches)
        }
      }

      // Modern browsers
      if (mediaQuery.addEventListener) {
        mediaQuery.addEventListener('change', handleChange)
      }
    }
  })

  return {
    theme,
    isDark,
    applyTheme,
    loadTheme,
  }
}
