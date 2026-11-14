import React, { useId } from 'react';

export interface SvgIconProps {
  className?: string;
  size?: number;
  title?: string;
}

/**
 * Custom hexagonal kiosk + prompt SVG logo mark for LogicKiosk.
 */
export const LogoMarkIcon: React.FC<SvgIconProps> = ({
  className = 'w-5 h-5',
  size,
  title = 'LogicKiosk',
}) => {
  const uid = useId().replace(/:/g, '');
  const gradId = `lk-logo-${uid}`;

  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 64 64"
      width={size}
      height={size}
      fill="none"
      className={className}
      role="img"
      aria-label={title}
    >
      <defs>
        <linearGradient id={gradId} x1="8" y1="6" x2="56" y2="58" gradientUnits="userSpaceOnUse">
          <stop offset="0%" stopColor="#22D3EE" />
          <stop offset="55%" stopColor="#06B6D4" />
          <stop offset="100%" stopColor="#A855F7" />
        </linearGradient>
      </defs>
      <path
        d="M32 4 L55 17 V47 L32 60 L9 47 V17 Z"
        fill="#0B0F19"
        stroke={`url(#${gradId})`}
        strokeWidth="3.5"
        strokeLinejoin="round"
      />
      <path
        d="M19 25 L28 32 L19 39"
        stroke="#22D3EE"
        strokeWidth="4"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path d="M31 39 H42" stroke="#10B981" strokeWidth="4" strokeLinecap="round" />
      <path
        d="M44 19 L46 23 L50 25 L46 27 L44 31 L42 27 L38 25 L42 23 Z"
        fill="#A855F7"
      />
    </svg>
  );
};

export const SandboxShieldIcon: React.FC<SvgIconProps> = ({
  className = 'w-5 h-5',
  size,
  title = 'Sandbox AST Segura',
}) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 32 32"
    width={size}
    height={size}
    fill="none"
    className={className}
    role="img"
    aria-label={title}
  >
    <path
      d="M16 3 L27 7.5 V15 C27 22 22.2 27.2 16 29.5 C9.8 27.2 5 22 5 15 V7.5 Z"
      fill="#0B0F19"
      stroke="#10B981"
      strokeWidth="2"
      strokeLinejoin="round"
    />
    <circle cx="16" cy="11" r="2" fill="#22D3EE" />
    <circle cx="11.5" cy="18" r="1.8" fill="#10B981" />
    <circle cx="20.5" cy="18" r="1.8" fill="#A855F7" />
    <path d="M14.8 12.5 L12.3 16.5 M17.2 12.5 L19.7 16.5" stroke="#94A3B8" strokeWidth="1.4" strokeLinecap="round" />
    <path d="M12.5 22.5 L15 24.8 L19.8 20.2" stroke="#34D399" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
  </svg>
);

export const SocraticMentorIcon: React.FC<SvgIconProps> = ({
  className = 'w-5 h-5',
  size,
  title = 'Mentor Socrático',
}) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 32 32"
    width={size}
    height={size}
    fill="none"
    className={className}
    role="img"
    aria-label={title}
  >
    <path
      d="M16 4 C10 4 6 8.5 6 14 C6 17.8 8 20.5 10.5 22.5 C11.2 23.2 11.7 24.2 12 25.5 H20 C20.3 24.2 20.8 23.2 21.5 22.5 C24 20.5 26 17.8 26 14 C26 8.5 22 4 16 4 Z"
      fill="#111827"
      stroke="#A855F7"
      strokeWidth="2"
      strokeLinejoin="round"
    />
    <path d="M12.5 28 H19.5" stroke="#94A3B8" strokeWidth="2" strokeLinecap="round" />
    <path
      d="M13 11.5 C13 9.5 14.5 8.5 16 8.5 C17.8 8.5 19 9.7 19 11.2 C19 13 16 13.8 16 16"
      stroke="#22D3EE"
      strokeWidth="2"
      strokeLinecap="round"
    />
    <circle cx="16" cy="19.2" r="1.3" fill="#A855F7" />
  </svg>
);

export const BossCrownIcon: React.FC<SvgIconProps> = ({
  className = 'w-5 h-5',
  size,
  title = 'Fase Boss',
}) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 32 32"
    width={size}
    height={size}
    fill="none"
    className={className}
    role="img"
    aria-label={title}
  >
    <path
      d="M5 22 L7 10 L12.5 15 L16 7 L19.5 15 L25 10 L27 22 Z"
      fill="#1E293B"
      stroke="#F59E0B"
      strokeWidth="2"
      strokeLinejoin="round"
    />
    <path d="M6 25.5 H26" stroke="#FBBF24" strokeWidth="2.2" strokeLinecap="round" />
    <circle cx="16" cy="17.5" r="1.8" fill="#06B6D4" />
  </svg>
);

export const LogicFragmentIcon: React.FC<SvgIconProps> = ({
  className = 'w-5 h-5',
  size,
  title = 'Fragmento de Lógica',
}) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 32 32"
    width={size}
    height={size}
    fill="none"
    className={className}
    role="img"
    aria-label={title}
  >
    <polygon
      points="16,3 25,11 22,26 16,29 10,26 7,11"
      fill="#0B0F19"
      stroke="#06B6D4"
      strokeWidth="2"
      strokeLinejoin="round"
    />
    <polygon points="16,3 25,11 16,16 7,11" fill="#22D3EE" fillOpacity="0.28" />
    <polygon points="16,16 25,11 22,26 16,29" fill="#A855F7" fillOpacity="0.28" />
    <path d="M16 3 V29 M7 11 L16 16 L25 11" stroke="#67E8F9" strokeWidth="1.2" />
  </svg>
);

export const WalletSealIcon: React.FC<SvgIconProps> = ({
  className = 'w-5 h-5',
  size,
  title = 'Certificado SHA-256',
}) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 32 32"
    width={size}
    height={size}
    fill="none"
    className={className}
    role="img"
    aria-label={title}
  >
    <path d="M11 20 L9 29 L13.5 26 L16 29 V21" fill="#10B981" fillOpacity="0.35" stroke="#10B981" strokeWidth="1.5" strokeLinejoin="round" />
    <path d="M21 20 L23 29 L18.5 26 L16 29 V21" fill="#06B6D4" fillOpacity="0.35" stroke="#06B6D4" strokeWidth="1.5" strokeLinejoin="round" />
    <circle cx="16" cy="13" r="9" fill="#0B0F19" stroke="#34D399" strokeWidth="2" />
    <path d="M12.2 13 L15 15.8 L20.2 10.5" stroke="#34D399" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" />
  </svg>
);

export const StreakFlameIcon: React.FC<SvgIconProps> = ({
  className = 'w-5 h-5',
  size,
  title = 'Ofensiva Diária',
}) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 32 32"
    width={size}
    height={size}
    fill="none"
    className={className}
    role="img"
    aria-label={title}
  >
    <path
      d="M17 3 C17 3 24.5 10 24.5 18 C24.5 24.2 20.5 29 16 29 C11 29 7.5 24.5 7.5 18.8 C7.5 14.5 10 11 12 9 C12 12 13.8 13.5 13.8 13.5 C13.8 8.5 17 3 17 3 Z"
      fill="#F59E0B"
      stroke="#FBBF24"
      strokeWidth="1.2"
    />
    <path
      d="M16.2 13 C16.2 13 20.5 17 20.5 21 C20.5 24.5 18.5 27 16 27 C13.5 27 11.5 24.5 11.5 21.2 C11.5 18.5 13.5 16 16.2 13 Z"
      fill="#FEF08A"
    />
  </svg>
);
