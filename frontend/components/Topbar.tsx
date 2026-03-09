'use client'

import { Bell, Search, Sun, Moon, LogOut } from 'lucide-react'
import { useTheme } from 'next-themes'
import { useRouter } from 'next/navigation'
import { useQuery } from '@tanstack/react-query'
import { useAuth } from '@/contexts/AuthContext'
import { notificationsApi } from '@/services/api'
import styles from './Topbar.module.scss'

export function Topbar() {
  const { theme, setTheme } = useTheme()
  const { user, logout } = useAuth()
  const router = useRouter()

  const { data: unreadData } = useQuery({
    queryKey: ['notifications', 'unread'],
    queryFn: notificationsApi.unreadCount,
    refetchInterval: 30_000,
    enabled: !!user,
  })

  const unread = unreadData?.unread_count ?? 0

  return (
    <header className={styles.topbar}>
      {/* Search */}
      <div className={styles.search}>
        <Search size={16} className={styles.searchIcon} />
        <input
          className={styles.searchInput}
          placeholder="Search students, courses, faculty..."
          type="search"
        />
      </div>

      {/* Actions */}
      <div className={styles.actions}>
        {/* Theme toggle */}
        <button
          className={styles.iconBtn}
          onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
          aria-label="Toggle theme"
        >
          {theme === 'dark' ? <Sun size={18} /> : <Moon size={18} />}
        </button>

        {/* Notifications */}
        <button
          className={styles.iconBtn}
          onClick={() => router.push('/notifications')}
          aria-label="Notifications"
        >
          <Bell size={18} />
          {unread > 0 && (
            <span className={styles.badge}>{unread > 99 ? '99+' : unread}</span>
          )}
        </button>

        {/* User avatar */}
        {user && (
          <div className={styles.userMenu}>
            <div className={styles.avatar}>
              {user.first_name[0]}{user.last_name[0]}
            </div>
            <div className={styles.userDetails}>
              <span className={styles.name}>{user.first_name} {user.last_name}</span>
              <span className={styles.role}>{user.role}</span>
            </div>
            <button className={styles.logoutBtn} onClick={logout} aria-label="Logout">
              <LogOut size={16} />
            </button>
          </div>
        )}
      </div>
    </header>
  )
}
