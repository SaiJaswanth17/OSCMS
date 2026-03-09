'use client'

import { useTheme } from 'next-themes'
import { useThemeColors } from '@/contexts/ThemeContext'
import { Sun, Moon, RefreshCw } from 'lucide-react'

export default function SettingsPage() {
  const { theme, setTheme } = useTheme()
  const { colors, updateColor, resetColors } = useThemeColors()

  return (
    <div className="animate-fade-in-up">
      <div className="page-header">
        <h1>Settings</h1>
        <p>Customize your OCMS experience</p>
      </div>

      {/* Appearance */}
      <div className="card" style={{ padding:'1.5rem', marginBottom:'1.5rem' }}>
        <h2 style={{ fontSize:'var(--font-size-lg)', marginBottom:'1.25rem' }}>Appearance</h2>

        <div style={{ display:'flex', flexDirection:'column', gap:'1.25rem' }}>
          {/* Theme */}
          <div>
            <label style={{ display:'block', fontWeight:500, fontSize:'0.875rem', color:'var(--secondary-font-color)', marginBottom:'0.625rem' }}>
              Color Mode
            </label>
            <div style={{ display:'flex', gap:'0.75rem' }}>
              {[
                { value:'dark',  label:'Dark',  icon:<Moon size={16}/> },
                { value:'light', label:'Light', icon:<Sun size={16}/> },
              ].map(opt => (
                <button
                  key={opt.value}
                  className={`btn ${theme === opt.value ? 'btn-primary' : 'btn-ghost'}`}
                  onClick={() => setTheme(opt.value)}
                >
                  {opt.icon} {opt.label}
                </button>
              ))}
            </div>
          </div>

          {/* Color customization */}
          <div>
            <div style={{ display:'flex', alignItems:'center', justifyContent:'space-between', marginBottom:'0.625rem' }}>
              <label style={{ fontWeight:500, fontSize:'0.875rem', color:'var(--secondary-font-color)' }}>
                Font & Accent Colors
              </label>
              <button className="btn btn-ghost btn-sm" onClick={resetColors}>
                <RefreshCw size={13} /> Reset
              </button>
            </div>
            <div style={{ display:'grid', gridTemplateColumns:'repeat(auto-fill, minmax(240px, 1fr))', gap:'1rem' }}>
              <ColorPicker
                label="Primary Text Color"
                value={colors.primaryFontColor}
                onChange={v => updateColor('primaryFontColor', v)}
              />
              <ColorPicker
                label="Secondary Text Color"
                value={colors.secondaryFontColor}
                onChange={v => updateColor('secondaryFontColor', v)}
              />
              <ColorPicker
                label="Accent Color"
                value={colors.accentColor}
                onChange={v => updateColor('accentColor', v)}
              />
            </div>
          </div>
        </div>
      </div>

      {/* Profile */}
      <div className="card" style={{ padding:'1.5rem' }}>
        <h2 style={{ fontSize:'var(--font-size-lg)', marginBottom:'1.25rem' }}>Profile & Security</h2>
        <div style={{ display:'grid', gridTemplateColumns:'1fr 1fr', gap:'1rem' }}>
          <div className="form-group">
            <label>First Name</label>
            <input className="input" placeholder="First name" />
          </div>
          <div className="form-group">
            <label>Last Name</label>
            <input className="input" placeholder="Last name" />
          </div>
          <div className="form-group">
            <label>Email</label>
            <input className="input" type="email" placeholder="email@institution.edu" />
          </div>
          <div className="form-group">
            <label>Phone</label>
            <input className="input" type="tel" placeholder="+1 234 567 8900" />
          </div>
        </div>
        <div style={{ marginTop:'1.25rem', paddingTop:'1.25rem', borderTop:'1px solid var(--border-subtle)' }}>
          <h3 style={{ fontSize:'0.95rem', marginBottom:'1rem' }}>Change Password</h3>
          <div style={{ display:'grid', gridTemplateColumns:'1fr 1fr', gap:'1rem' }}>
            <div className="form-group">
              <label>Current Password</label>
              <input className="input" type="password" placeholder="••••••••" />
            </div>
            <div className="form-group">
              <label>New Password</label>
              <input className="input" type="password" placeholder="••••••••" />
            </div>
          </div>
        </div>
        <div style={{ marginTop:'1.25rem', display:'flex', justifyContent:'flex-end', gap:'0.75rem' }}>
          <button className="btn btn-ghost">Cancel</button>
          <button className="btn btn-primary">Save Changes</button>
        </div>
      </div>
    </div>
  )
}

function ColorPicker({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  return (
    <div style={{ display:'flex', alignItems:'center', gap:'0.75rem', padding:'0.75rem', background:'var(--bg-input)', borderRadius:'var(--radius-md)', border:'1px solid var(--border-default)' }}>
      <input
        type="color"
        value={value}
        onChange={e => onChange(e.target.value)}
        style={{ width:32, height:32, border:'none', background:'none', cursor:'pointer', padding:0 }}
      />
      <div>
        <div style={{ fontSize:'0.8rem', fontWeight:600 }}>{label}</div>
        <div style={{ fontSize:'0.72rem', color:'var(--muted-font-color)', fontFamily:'var(--font-mono)' }}>{value}</div>
      </div>
    </div>
  )
}
