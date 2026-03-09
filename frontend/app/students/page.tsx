'use client'

import { useQuery } from '@tanstack/react-query'
import { studentsApi } from '@/services/api'
import { Search, UserPlus, Trash2, Eye } from 'lucide-react'
import { useState } from 'react'
import Link from 'next/link'

interface Student {
  id: string; student_id: string; first_name: string; last_name: string
  email: string; department_name?: string; enrollment_year: number
  current_semester: number; gpa?: number; is_active: boolean
}
interface Paginated { data: Student[]; meta: { total: number; page: number; total_pages: number } }

export default function StudentsPage() {
  const [search, setSearch] = useState('')
  const [page, setPage] = useState(1)

  const { data, isLoading } = useQuery<Paginated>({
    queryKey: ['students', page],
    queryFn: () => studentsApi.list({ page: String(page), limit: '20' }) as Promise<Paginated>,
  })

  const students = data?.data ?? []

  return (
    <div className="animate-fade-in-up">
      <div className="page-header flex items-center justify-between" style={{ display:'flex', alignItems:'center', justifyContent:'space-between' }}>
        <div>
          <h1>Students</h1>
          <p>Manage all enrolled students ({data?.meta.total ?? 0} total)</p>
        </div>
        <Link href="/students/new" className="btn btn-primary">
          <UserPlus size={16} /> Add Student
        </Link>
      </div>

      {/* Filters */}
      <div className="card" style={{ padding:'1rem', marginBottom:'1.5rem', display:'flex', gap:'0.75rem' }}>
        <div style={{ position:'relative', flex:1 }}>
          <Search size={15} style={{ position:'absolute', left:'0.75rem', top:'50%', transform:'translateY(-50%)', color:'var(--muted-font-color)' }} />
          <input
            className="input"
            style={{ paddingLeft:'2.25rem' }}
            placeholder="Search by name, email, or student ID..."
            value={search}
            onChange={e => setSearch(e.target.value)}
          />
        </div>
      </div>

      {/* Table */}
      <div className="card">
        {isLoading ? (
          <div style={{ padding:'3rem', textAlign:'center', color:'var(--secondary-font-color)' }}>Loading students...</div>
        ) : (
          <div className="table-wrapper">
            <table className="table">
              <thead>
                <tr>
                  <th>Student</th>
                  <th>ID</th>
                  <th>Department</th>
                  <th>Semester</th>
                  <th>GPA</th>
                  <th>Status</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {students
                  .filter(s =>
                    !search ||
                    `${s.first_name} ${s.last_name} ${s.email} ${s.student_id}`
                      .toLowerCase().includes(search.toLowerCase())
                  )
                  .map(s => (
                    <tr key={s.id}>
                      <td>
                        <div style={{ display:'flex', alignItems:'center', gap:'0.625rem' }}>
                          <div style={{
                            width:32, height:32, borderRadius:'50%',
                            background:'var(--color-primary)', color:'#fff',
                            display:'flex', alignItems:'center', justifyContent:'center',
                            fontSize:'0.7rem', fontWeight:700, flexShrink:0
                          }}>
                            {s.first_name[0]}{s.last_name[0]}
                          </div>
                          <div>
                            <div style={{ fontWeight:600, fontSize:'0.875rem' }}>{s.first_name} {s.last_name}</div>
                            <div style={{ color:'var(--muted-font-color)', fontSize:'0.75rem' }}>{s.email}</div>
                          </div>
                        </div>
                      </td>
                      <td><code style={{ fontSize:'0.8rem', color:'var(--color-accent)' }}>{s.student_id}</code></td>
                      <td>{s.department_name ?? '—'}</td>
                      <td>Sem {s.current_semester}</td>
                      <td>
                        <span style={{ fontWeight:700, color: (s.gpa ?? 0) >= 3.5 ? 'var(--color-success)' : (s.gpa ?? 0) >= 2.5 ? 'var(--color-warning)' : 'var(--color-danger)' }}>
                          {s.gpa?.toFixed(2) ?? 'N/A'}
                        </span>
                      </td>
                      <td>
                        <span className={`badge ${s.is_active ? 'badge-success' : 'badge-danger'}`}>
                          {s.is_active ? 'Active' : 'Inactive'}
                        </span>
                      </td>
                      <td>
                        <div style={{ display:'flex', gap:'0.375rem' }}>
                          <Link href={`/students/${s.id}`} className="btn btn-ghost btn-sm"><Eye size={14} /></Link>
                        </div>
                      </td>
                    </tr>
                  ))}
              </tbody>
            </table>
          </div>
        )}

        {/* Pagination */}
        {data && data.meta.total_pages > 1 && (
          <div style={{ display:'flex', justifyContent:'center', gap:'0.5rem', padding:'1rem', borderTop:'1px solid var(--border-default)' }}>
            <button className="btn btn-ghost btn-sm" onClick={() => setPage(p => Math.max(1, p-1))} disabled={page === 1}>Previous</button>
            <span style={{ display:'flex', alignItems:'center', fontSize:'0.875rem', color:'var(--secondary-font-color)' }}>
              Page {data.meta.page} of {data.meta.total_pages}
            </span>
            <button className="btn btn-ghost btn-sm" onClick={() => setPage(p => Math.min(data.meta.total_pages, p+1))} disabled={page === data.meta.total_pages}>Next</button>
          </div>
        )}
      </div>
    </div>
  )
}
