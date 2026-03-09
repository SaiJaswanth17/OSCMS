'use client'

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { notificationsApi } from '@/services/api'
import { Bell, Check, Info, AlertTriangle, CheckCircle, Loader } from 'lucide-react'

interface Notification {
  id: string; title: string; message: string
  type: string; is_read: boolean; created_at: string; link?: string
}
interface Paginated { data: Notification[]; meta: { total: number; } }

const typeIcons: Record<string, React.ReactNode> = {
  INFO:        <Info size={16} />,
  WARNING:     <AlertTriangle size={16} />,
  SUCCESS:     <CheckCircle size={16} />,
  EXAM:        <Bell size={16} />,
  ATTENDANCE:  <Check size={16} />,
  GRADE:       <CheckCircle size={16} />,
  ANNOUNCEMENT:<Bell size={16} />,
}

const typeBadge: Record<string, string> = {
  INFO: 'badge-info', WARNING: 'badge-warning', SUCCESS: 'badge-success',
  EXAM: 'badge-primary', ATTENDANCE: 'badge-primary', GRADE: 'badge-success', ANNOUNCEMENT: 'badge-info',
}

export default function NotificationsPage() {
  const qc = useQueryClient()

  const { data, isLoading } = useQuery<Paginated>({
    queryKey: ['notifications'],
    queryFn: () => notificationsApi.list() as Promise<Paginated>,
  })

  const { mutate: markAllRead, isPending } = useMutation({
    mutationFn: notificationsApi.markAllRead,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['notifications'] })
      qc.invalidateQueries({ queryKey: ['notifications', 'unread'] })
    },
  })

  const notifs = data?.data ?? []

  return (
    <div className="animate-fade-in-up">
      <div style={{ display:'flex', alignItems:'center', justifyContent:'space-between', marginBottom:'2rem' }}>
        <div>
          <h1>Notifications</h1>
          <p style={{ color:'var(--secondary-font-color)', marginTop:4 }}>
            {data?.meta.total ?? 0} total notifications
          </p>
        </div>
        <button
          className="btn btn-ghost"
          onClick={() => markAllRead()}
          disabled={isPending}
        >
          {isPending ? <Loader size={15} /> : <Check size={15} />}
          Mark all as read
        </button>
      </div>

      <div className="card" style={{ overflow:'hidden' }}>
        {isLoading ? (
          <div style={{ padding:'3rem', textAlign:'center', color:'var(--secondary-font-color)' }}>Loading notifications...</div>
        ) : notifs.length === 0 ? (
          <div style={{ padding:'3rem', textAlign:'center', color:'var(--muted-font-color)' }}>
            <Bell size={40} style={{ margin:'0 auto 1rem', opacity:0.3 }} />
            <p>No notifications yet</p>
          </div>
        ) : (
          <div>
            {notifs.map((n, i) => (
              <div
                key={n.id}
                style={{
                  display:'flex', gap:'1rem', padding:'1.25rem',
                  borderBottom: i < notifs.length-1 ? '1px solid var(--border-subtle)' : 'none',
                  background: n.is_read ? 'transparent' : 'var(--color-primary-soft)',
                  transition:'background var(--transition-fast)',
                }}
              >
                <div style={{
                  width:36, height:36, borderRadius:'var(--radius-md)',
                  background:'var(--bg-input)', border:'1px solid var(--border-default)',
                  display:'flex', alignItems:'center', justifyContent:'center',
                  color:'var(--color-primary)', flexShrink:0
                }}>
                  {typeIcons[n.type] ?? <Bell size={16} />}
                </div>
                <div style={{ flex:1, minWidth:0 }}>
                  <div style={{ display:'flex', alignItems:'flex-start', justifyContent:'space-between', gap:'0.5rem', marginBottom:4 }}>
                    <span style={{ fontWeight:600, fontSize:'0.875rem' }}>{n.title}</span>
                    <span className={`badge ${typeBadge[n.type] ?? 'badge-info'}`} style={{ flexShrink:0 }}>{n.type}</span>
                  </div>
                  <p style={{ fontSize:'0.8rem', color:'var(--secondary-font-color)', lineHeight:1.5 }}>{n.message}</p>
                  <p style={{ fontSize:'0.72rem', color:'var(--muted-font-color)', marginTop:6 }}>
                    {new Date(n.created_at).toLocaleString()}
                  </p>
                </div>
                {!n.is_read && (
                  <div style={{ width:8, height:8, borderRadius:'50%', background:'var(--color-primary)', flexShrink:0, marginTop:6 }} />
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
