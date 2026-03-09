import type { Metadata } from 'next'
import { Inter } from 'next/font/google'
import { Providers } from './providers'
import '@/styles/globals.scss'

const inter = Inter({
  subsets: ['latin'],
  variable: '--font-sans',
  display: 'swap',
})

export const metadata: Metadata = {
  title: {
    default: 'OCMS — Open College Management System',
    template: '%s | OCMS',
  },
  description:
    'A modern, open-source college management system for managing students, faculty, courses, attendance, and more.',
  keywords: ['college management', 'student portal', 'academic system', 'open source'],
  openGraph: {
    title: 'OCMS — Open College Management System',
    description: 'Modern academic lifecycle management platform',
    type: 'website',
  },
}

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={inter.variable}>
        <Providers>{children}</Providers>
      </body>
    </html>
  )
}
