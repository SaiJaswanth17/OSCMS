'use client'

import { usePathname } from 'next/navigation'
import Link from 'next/link'
import {
  LayoutDashboard,
  Users,
  GraduationCap,
  BookOpen,
  ClipboardList,
  FileText,
  Bell,
  Settings,
  ChevronLeft,
  ChevronRight,
} from 'lucide-react'
import { useState } from 'react'
import { useAuth } from '@/contexts/AuthContext'
import styles from './Sidebar.module.scss'

const NAV_ITEMS = [
  { href: '/dashboard',     label: 'Dashboard',    icon: LayoutDashboard, roles: ['ADMIN', 'STUDENT', 'FACULTY', 'DEPT_HEAD'] },
  { href: '/students',      label: 'Students',     icon: Users,            roles: ['ADMIN', 'DEPT_HEAD', 'FACULTY'] },
  { href: '/faculty',       label: 'Faculty',      icon: GraduationCap,   roles: ['ADMIN', 'DEPT_HEAD'] },
  { href: '/courses',       label: 'Courses',      icon: BookOpen,         roles: ['ADMIN', 'FACULTY', 'STUDENT', 'DEPT_HEAD'] },
  { href: '/attendance',    label: 'Attendance',   icon: ClipboardList,    roles: ['ADMIN', 'FACULTY', 'STUDENT'] },
  { href: '/exams',         label: 'Exams & Results', icon: FileText,      roles: ['ADMIN', 'FACULTY', 'STUDENT'] },
  { href: '/notifications', label: 'Notifications', icon: Bell,            roles: ['ADMIN', 'STUDENT', 'FACULTY', 'DEPT_HEAD'] },
  { href: '/settings',      label: 'Settings',     icon: Settings,         roles: ['ADMIN', 'STUDENT', 'FACULTY', 'DEPT_HEAD'] },
]

export function Sidebar() {
  const pathname = usePathname()
  const { user } = useAuth()
  const [collapsed, setCollapsed] = useState(false)

  const filteredItems = NAV_ITEMS.filter(
    item => user?.role && item.roles.includes(user.role)
  )

  return (
    <aside className={`${styles.sidebar} ${collapsed ? styles.collapsed : ''}`}>
      {/* Logo */}
      <div className={styles.logo}>
        <div className={styles.logoIcon}>
          <span>🎓</span>
        </div>
        {!collapsed && <span className={styles.logoText}>OCMS</span>}
      </div>

      {/* Navigation */}
      <nav className={styles.nav}>
        {filteredItems.map(item => {
          const isActive = pathname === item.href || pathname.startsWith(`${item.href}/`)
          const Icon = item.icon
          return (
            <Link
              key={item.href}
              href={item.href}
              className={`${styles.navItem} ${isActive ? styles.active : ''}`}
              title={collapsed ? item.label : undefined}
            >
              <Icon size={20} className={styles.navIcon} />
              {!collapsed && <span className={styles.navLabel}>{item.label}</span>}
              {isActive && <span className={styles.activePill} />}
            </Link>
          )
        })}
      </nav>

      {/* Collapse toggle */}
      <button
        className={styles.collapseBtn}
        onClick={() => setCollapsed(c => !c)}
        aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
      >
        {collapsed ? <ChevronRight size={16} /> : <ChevronLeft size={16} />}
      </button>

      {/* User pill at bottom */}
      {user && !collapsed && (
        <div className={styles.userPill}>
          <div className={styles.avatar}>
            {user.first_name[0]}{user.last_name[0]}
          </div>
          <div className={styles.userInfo}>
            <span className={styles.userName}>{user.first_name} {user.last_name}</span>
            <span className={`badge badge-primary ${styles.roleBadge}`}>{user.role}</span>
          </div>
        </div>
      )}
    </aside>
  )
}
