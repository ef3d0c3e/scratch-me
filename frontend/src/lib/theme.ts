import type { ThemeId } from './types.ts';

const YEAR = 60 * 60 * 24 * 365;
export function setCookie(name: string, value: string) {
	document.cookie = `${name}=${encodeURIComponent(value)}; Path=/; Max-Age=${YEAR}; SameSite=Lax`;
}

export const THEMES: ThemeId[] = [
	'midnight',
	'sunset',
	'mint',
	'ocean',
	'candy',
	'arcade',
	'neon',
	'royal',
	'forest',
	'champion'
];
export const parseSkin = (v: string | undefined): ThemeId =>
	THEMES.find((t) => t === v) ?? 'midnight';
