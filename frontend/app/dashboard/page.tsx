'use client'

import { useAuth } from '@/contexts/AuthContext'
import { useQuery } from '@tanstack/react-query'
import { studentsApi, facultyApi, coursesApi } from '@/services/api'
import {
  Users, GraduationCap, BookOpen, Building2,
  TrendingUp, Activity, BarChart3, ArrowUpRight
} from 'lucide-react'

// ─── Types (light, good enough for demo data) ─────────────────────

interface PaginatedMeta { total: number; page: number; limit: number; total_pages: number }
interface Paginated<T> { data: T[]; meta: PaginatedMeta }

export default function DashboardPage() {
  const { user } = useAuth()

  const { data: studentsData } = useQuery<Paginated<unknown>>({
    queryKey: ['students', 'summary'],
    queryFn: () => studentsApi.list({ limit: '1' }) as Promise<Paginated<unknown>>,
    enabled: user?.role === 'ADMIN' || user?.role === 'DEPT_HEAD',
  })

  const { data: facultyData } = useQuery<Paginated<unknown>>({
    queryKey: ['faculty', 'summary'],
    queryFn: () => facultyApi.list({ limit: '1' }) as Promise<Paginated<unknown>>,
    enabled: user?.role === 'ADMIN',
  })

  const { data: coursesData } = useQuery<Paginated<unknown>>({
    queryKey: ['courses', 'summary'],
    queryFn: () => coursesApi.list({ limit: '1' }) as Promise<Paginated<unknown>>,
    enabled: !!user,
  })

  const isAdmin = user?.role === 'ADMIN'

  return (
    <div className="animate-fade-in-up">
      {/* Page header */}
      <div className="page-header">
        <h1>👋 Welcome back, {user?.first_name}</h1>
        <p>Here's what's happening at your institution today.</p>
      </div>

      {/* Stats grid */}
      {isAdmin && (
        <div className="grid-cols-4" style={{ marginBottom: '2rem' }}>
          <StatCard
            icon={<Users size={22} />}
            label="Total Students"
            value={studentsData?.meta.total ?? '—'}
            change="+12 this month"
            positive
            color="primary"
          />
          <StatCard
            icon={<GraduationCap size={22} />}
            label="Total Faculty"
            value={facultyData?.meta.total ?? '—'}
            change="Active staff"
            positive
            color="accent"
          />
          <StatCard
            icon={<BookOpen size={22} />}
            label="Active Courses"
            value={coursesData?.meta.total ?? '—'}
            change="This semester"
            positive
            color="success"
          />
          <StatCard
            icon={<Building2 size={22} />}
            label="Departments"
            value="12"
            change="Fully operational"
            positive
            color="info"
          />
        </div>
      )}

      {/* Student dashboard */}
      {user?.role === 'STUDENT' && (
        <div className="grid-cols-4" style={{ marginBottom: '2rem' }}>
          <StatCard icon={<BarChart3 size={22} />} label="GPA" value="3.74" change="↑ from 3.68" positive color="success" />
          <StatCard icon={<BookOpen size={22} />}  label="Enrolled Courses" value="6" change="Current semester" positive color="primary" />
          <StatCard icon={<Activity size={22} />}  label="Attendance Rate" value="87%" change="Above minimum" positive color="accent" />
          <StatCard icon={<TrendingUp size={22} />} label="Credits Earned" value="72" change="of 120 required" positive color="info" />
        </div>
      )}

      {/* Quick actions */}
      <div className="grid-cols-2">
        <div className="card" style={{ padding: '1.5rem' }}>
          <div className="flex items-center justify-between" style={{ marginBottom: '1rem' }}>
            <h3 style={{ fontSize: 'var(--font-size-base)' }}>Quick Actions</h3>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.625rem' }}>
            {isAdmin && (
              <>
                <QuickAction label="Add New Student" href="/students/new" />
                <QuickAction label="Create Course" href="/courses/new" />
                <QuickAction label="Schedule Exam" href="/exams/new" />
                <QuickAction label="Generate Report" href="/reports" />
              </>
            )}
            {user?.role === 'FACULTY' && (
              <>
                <QuickAction label="Mark Attendance" href="/attendance/mark" />
                <QuickAction label="Upload Results" href="/exams/upload" />
                <QuickAction label="View My Courses" href="/courses" />
              </>
            )}
            {user?.role === 'STUDENT' && (
              <>
                <QuickAction label="View My Grades" href="/exams" />
                <QuickAction label="Check Attendance" href="/attendance" />
                <QuickAction label="Browse Courses" href="/courses" />
                <QuickAction label="Download Transcript" href="/exams/transcript" />
              </>
            )}
          </div>
        </div>

        <div className="card" style={{ padding: '1.5rem' }}>
          <div className="flex items-center justify-between" style={{ marginBottom: '1rem' }}>
            <h3 style={{ fontSize: 'var(--font-size-base)' }}>Recent Activity</h3>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            {MOCK_ACTIVITY.map((item, i) => (
              <div key={i} className="flex items-center gap-4">
                <div style={{
                  width: 36, height: 36, borderRadius: 'var(--radius-md)',
                  background: item.bg, display: 'flex', alignItems: 'center',
                  justifyContent: 'center', fontSize: '1rem', flexShrink: 0
                }}>
                  {item.icon}
                </div>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <p style={{ fontSize: 'var(--font-size-sm)', fontWeight: 500 }}>{item.text}</p>
                  <p className="text-muted text-xs">{item.time}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}

// ─── Sub-components ────────────────────────────

function StatCard({ icon, label, value, change, positive, color }: {
  icon: React.ReactNode
  label: string
  value: string | number
  change: string
  positive?: boolean
  color: 'primary' | 'accent' | 'success' | 'info'
}) {
  const colorMap = {
    primary: { bg: 'var(--color-primary-soft)', text: 'var(--color-primary)' },
    accent:  { bg: 'var(--color-accent-soft)',  text: 'var(--color-accent)' },
    success: { bg: 'var(--color-success-soft)', text: 'var(--color-success)' },
    info:    { bg: 'var(--color-info-soft)',    text: 'var(--color-info)' },
  }
  const c = colorMap[color]

  return (
    <div className="stats-card">
      <div className="stats-icon" style={{ background: c.bg, color: c.text }}>
        {icon}
      </div>
      <div className="stats-value">{value}</div>
      <div className="stats-label">{label}</div>
      <div className={`stats-change ${positive ? 'positive' : 'negative'}`}>
        <ArrowUpRight size={12} /> {change}
      </div>
    </div>
  )
}

function QuickAction({ label, href }: { label: string; href: string }) {
  return (
    <a
      href={href}
      style={{
        display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        padding: '0.625rem 0.875rem',
        background: 'var(--bg-input)', border: '1px solid var(--border-default)',
        borderRadius: 'var(--radius-md)',
        color: 'var(--primary-font-color)', fontSize: 'var(--font-size-sm)',
        fontWeight: 500, textDecoration: 'none',
        transition: 'all var(--transition-fast)',
      }}
      onMouseEnter={e => {
        const el = e.currentTarget as HTMLAnchorElement
        el.style.borderColor = 'var(--color-primary)'
        el.style.color = 'var(--color-primary)'
      }}
      onMouseLeave={e => {
        const el = e.currentTarget as HTMLAnchorElement
        el.style.borderColor = 'var(--border-default)'
        el.style.color = 'var(--primary-font-color)'
      }}
    >
      {label}
      <ArrowUpRight size={14} />
    </a>
  )
}

const MOCK_ACTIVITY = [
  { icon: '👤', bg: 'var(--color-primary-soft)',  text: 'New student enrolled: Jane Doe',          time: '2 min ago' },
  { icon: '📚', bg: 'var(--color-accent-soft)',   text: 'CS301 Midterm scheduled',                 time: '1 hr ago' },
  { icon: '✅', bg: 'var(--color-success-soft)',  text: 'Attendance marked for CS201',             time: '3 hrs ago' },
  { icon: '📊', bg: 'var(--color-info-soft)',     text: 'Q3 results published for Physics 101',   time: '5 hrs ago' },
  { icon: '🔔', bg: 'var(--color-warning-soft)',  text: 'System maintenance scheduled for Friday', time: '1 day ago' },
]
