const BASE_URL = process.env.NEXT_PUBLIC_API_URL ?? ''

async function request<T>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  // Get token from cookie
  const token = document.cookie
    .split('; ')
    .find(row => row.startsWith('ocms_access_token='))
    ?.split('=')[1]

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string>),
  }

  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }

  const res = await fetch(`${BASE_URL}${path}`, { ...options, headers })

  if (!res.ok) {
    const error = await res.json().catch(() => ({ error: { message: 'Request failed' } }))
    throw new Error(error.error?.message ?? 'Request failed')
  }

  // 204 No Content
  if (res.status === 204) return undefined as T

  return res.json()
}

// ─── Auth ─────────────────────────────────────

export const authApi = {
  login: (email: string, password: string) =>
    request<{ access_token: string; user: unknown }>('/api/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    }),
  me: () => request<unknown>('/api/auth/me'),
  logout: () => request<void>('/api/auth/logout', { method: 'POST' }),
}

// ─── Students ─────────────────────────────────

export const studentsApi = {
  list: (params?: Record<string, string>) => {
    const qs = params ? '?' + new URLSearchParams(params).toString() : ''
    return request<unknown>(`/api/students${qs}`)
  },
  get: (id: string) => request<unknown>(`/api/students/${id}`),
  delete: (id: string) => request<void>(`/api/students/${id}`, { method: 'DELETE' }),
}

// ─── Faculty ──────────────────────────────────

export const facultyApi = {
  list: (params?: Record<string, string>) => {
    const qs = params ? '?' + new URLSearchParams(params).toString() : ''
    return request<unknown>(`/api/faculty${qs}`)
  },
  get: (id: string) => request<unknown>(`/api/faculty/${id}`),
}

// ─── Courses ──────────────────────────────────

export const coursesApi = {
  list: (params?: Record<string, string>) => {
    const qs = params ? '?' + new URLSearchParams(params).toString() : ''
    return request<unknown>(`/api/courses${qs}`)
  },
  create: (body: unknown) =>
    request<{ id: string }>('/api/courses', {
      method: 'POST',
      body: JSON.stringify(body),
    }),
  enroll: (courseId: string) =>
    request<void>(`/api/courses/${courseId}/enroll`, { method: 'POST' }),
}

// ─── Attendance ───────────────────────────────

export const attendanceApi = {
  mark: (body: unknown) =>
    request<{ session_id: string }>('/api/attendance/mark', {
      method: 'POST',
      body: JSON.stringify(body),
    }),
  getStudentAttendance: (studentId: string) =>
    request<unknown[]>(`/api/attendance/student/${studentId}`),
}

// ─── Results ──────────────────────────────────

export const resultsApi = {
  upload: (body: unknown) =>
    request<void>('/api/results/upload', {
      method: 'POST',
      body: JSON.stringify(body),
    }),
  getStudentResults: (studentId: string) =>
    request<unknown[]>(`/api/results/student/${studentId}`),
  publish: (examId: string) =>
    request<void>(`/api/results/exam/${examId}/publish`, { method: 'POST' }),
}

// ─── Notifications ────────────────────────────

export const notificationsApi = {
  list: (params?: Record<string, string>) => {
    const qs = params ? '?' + new URLSearchParams(params).toString() : ''
    return request<unknown>(`/api/notifications${qs}`)
  },
  markAllRead: () =>
    request<void>('/api/notifications/read-all', { method: 'POST' }),
  unreadCount: () =>
    request<{ unread_count: number }>('/api/notifications/unread-count'),
}
