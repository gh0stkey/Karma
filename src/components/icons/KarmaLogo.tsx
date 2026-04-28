const KarmaLogo = ({
  width,
  height,
  className,
}: {
  width?: number;
  height?: number;
  className?: string;
}) => {
  const stroke = "var(--color-logo-stroke)";
  const primary = "var(--color-logo-primary)";

  return (
    <svg
      width={width}
      height={height}
      className={className}
      viewBox="0 0 512 512"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M128 48h76v162L392 48v104L248 256l144 104v104L204 302v162h-76z"
        fill={stroke}
      />
      <rect x="116" y="244" width="288" height="24" rx="4" fill={primary} />
    </svg>
  );
};

export default KarmaLogo;
