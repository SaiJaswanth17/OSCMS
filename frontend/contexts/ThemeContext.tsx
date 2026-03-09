'use client'

import { createContext, useContext, useState, useEffect, ReactNode, useCallback } from 'react'

interface ThemeColors {
  primaryFontColor: string
  secondaryFontColor: string
  accentColor: string
}

interface ThemeContextType {
  colors: ThemeColors
  updateColor: (key: keyof ThemeColors, value: string) => void
  resetColors: () => void
}

const DEFAULT_COLORS: ThemeColors = {
  primaryFontColor: '#f1f5f9',
  secondaryFontColor: '#94a3b8',
  accentColor: '#4f6ef7',
}

const STORAGE_KEY = 'ocms_theme_colors'
const ThemeContext = createContext<ThemeContextType | null>(null)

export function ThemeCustomProvider({ children }: { children: ReactNode }) {
  const [colors, setColors] = useState<ThemeColors>(DEFAULT_COLORS)

  // Load saved colors
  useEffect(() => {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved) {
      const parsed = JSON.parse(saved) as ThemeColors
      setColors(parsed)
      applyColors(parsed)
    }
  }, [])

  const applyColors = (c: ThemeColors) => {
    const root = document.documentElement
    root.style.setProperty('--primary-font-color',   c.primaryFontColor)
    root.style.setProperty('--secondary-font-color', c.secondaryFontColor)
    root.style.setProperty('--accent-color',         c.accentColor)
  }

  const updateColor = useCallback((key: keyof ThemeColors, value: string) => {
    const next = { ...colors, [key]: value }
    setColors(next)
    applyColors(next)
    localStorage.setItem(STORAGE_KEY, JSON.stringify(next))
  }, [colors])

  const resetColors = useCallback(() => {
    setColors(DEFAULT_COLORS)
    applyColors(DEFAULT_COLORS)
    localStorage.removeItem(STORAGE_KEY)
  }, [])

  return (
    <ThemeContext.Provider value={{ colors, updateColor, resetColors }}>
      {children}
    </ThemeContext.Provider>
  )
}

export function useThemeColors() {
  const ctx = useContext(ThemeContext)
  if (!ctx) throw new Error('useThemeColors must be inside ThemeCustomProvider')
  return ctx
}
