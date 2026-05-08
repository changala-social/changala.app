interface AuthorNameProps {
  did: string;
}

export function AuthorName({ did }: AuthorNameProps) {
  // In MVP, just truncate the DID. Full DID resolution via atproto-ui in M3.
  const short = did.startsWith('did:plc:')
    ? `@${did.slice(8, 15)}...`
    : did.length > 20
    ? `${did.slice(0, 18)}...`
    : did;

  return (
    <span className="text-sm text-text-secondary font-mono" title={did}>
      {short}
    </span>
  );
}
