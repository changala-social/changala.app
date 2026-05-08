export function LoadingSpinner({ size = 'md' }: { size?: 'sm' | 'md' | 'lg' }) {
  const sizeClasses = { sm: 'w-4 h-4', md: 'w-8 h-8', lg: 'w-12 h-12' };
  return (
    <div className="flex justify-center py-8">
      <div className={`${sizeClasses[size]} animate-spin rounded-full border-2 border-border border-t-academic`} />
    </div>
  );
}
