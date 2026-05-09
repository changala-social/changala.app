import { useState } from 'react';
import { useVerifyEmail } from '../../hooks/useIdentity';
import { useAuth } from '../../context/AuthContext';
import { ErrorMessage } from './ErrorMessage';

interface EmailVerificationProps {
  onVerified?: () => void;
}

export function EmailVerification({ onVerified }: EmailVerificationProps) {
  const { did } = useAuth();
  const verifyEmail = useVerifyEmail();

  const [email, setEmail] = useState('');
  const [otp, setOtp] = useState('');
  const [step, setStep] = useState<'email' | 'otp' | 'done'>('email');
  const [error, setError] = useState<string | null>(null);

  const handleRequestOtp = (e: React.FormEvent) => {
    e.preventDefault();
    if (!did || !email.trim()) return;
    setError(null);
    verifyEmail.mutate(
      { did, email: email.trim() },
      {
        onSuccess: (data) => {
          if (data.status === 'otpSent') setStep('otp');
        },
        onError: (err) => setError(err.message),
      }
    );
  };

  const handleVerifyOtp = (e: React.FormEvent) => {
    e.preventDefault();
    if (!did || !otp.trim()) return;
    setError(null);
    verifyEmail.mutate(
      { did, email: email.trim(), otp: otp.trim() },
      {
        onSuccess: (data) => {
          if (data.status === 'verified') {
            setStep('done');
            onVerified?.();
          }
        },
        onError: (err) => setError(err.message),
      }
    );
  };

  if (step === 'done') {
    return (
      <div className="rounded-lg border border-live/30 bg-live/5 p-4 text-center">
        <p className="text-sm font-medium text-live">Email verified! You now have institution access.</p>
      </div>
    );
  }

  return (
    <div className="rounded-lg border border-border bg-surface p-4 space-y-4">
      <h3 className="text-sm font-semibold text-text">Verify Institution Email</h3>
      <p className="text-xs text-text-muted">
        Verify your institution email to enroll in courses and participate in sessions.
      </p>

      {step === 'email' && (
        <form onSubmit={handleRequestOtp} className="space-y-3">
          <input
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="student@nitc.ac.in"
            className="w-full rounded-md border border-border bg-surface-alt px-3 py-2 text-sm text-text placeholder:text-text-muted focus:outline-none focus:ring-2 focus:ring-academic/50"
          />
          {error && <ErrorMessage message={error} />}
          <button
            type="submit"
            disabled={verifyEmail.isPending || !email.trim()}
            className="w-full rounded-md bg-academic px-3 py-2 text-sm font-medium text-white hover:bg-academic/90 disabled:opacity-50 transition"
          >
            {verifyEmail.isPending ? 'Sending...' : 'Send Verification Code'}
          </button>
        </form>
      )}

      {step === 'otp' && (
        <form onSubmit={handleVerifyOtp} className="space-y-3">
          <p className="text-xs text-text-secondary">Code sent to <strong>{email}</strong></p>
          <input
            type="text"
            value={otp}
            onChange={(e) => setOtp(e.target.value)}
            placeholder="Enter 6-digit code"
            maxLength={6}
            className="w-full rounded-md border border-border bg-surface-alt px-3 py-2 text-sm text-text text-center tracking-widest font-mono placeholder:text-text-muted focus:outline-none focus:ring-2 focus:ring-academic/50"
          />
          {error && <ErrorMessage message={error} />}
          <button
            type="submit"
            disabled={verifyEmail.isPending || otp.length < 6}
            className="w-full rounded-md bg-academic px-3 py-2 text-sm font-medium text-white hover:bg-academic/90 disabled:opacity-50 transition"
          >
            {verifyEmail.isPending ? 'Verifying...' : 'Verify'}
          </button>
          <button
            type="button"
            onClick={() => { setStep('email'); setOtp(''); setError(null); }}
            className="w-full text-xs text-text-muted hover:text-text"
          >
            Use a different email
          </button>
        </form>
      )}
    </div>
  );
}
