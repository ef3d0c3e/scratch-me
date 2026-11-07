/**
 * How sealed tickets look, and the paper of each tier. A seal's theme tints
 * the ticket and scatters its motifs over it; its wax badge says what it does.
 */

import type { I18n } from './i18n/index.svelte.ts';
import type { Seal, SealTheme } from './types.ts';

export const SEAL_LOOK: Record<
	SealTheme,
	{ icon: string; wax: string; paper: string; motifs: string[] }
> = {
	sunrise: { icon: '🌅', wax: '#f97316', paper: '#ffe1c4', motifs: ['☀️', '🌤️'] },
	beach: { icon: '🏖️', wax: '#0ea5e9', paper: '#fbe7c0', motifs: ['🌴', '🐚', '🌊', '🦀'] },
	casino: { icon: '🎰', wax: '#b91c1c', paper: '#d9efe0', motifs: ['♠', '♥', '♦', '♣'] },
	diamond: { icon: '💎', wax: '#2563eb', paper: '#e3f2ff', motifs: ['💎', '✨'] },
	cosmos: { icon: '🪐', wax: '#6d28d9', paper: '#e6defc', motifs: ['🪐', '✦', '★', '☄️'] }
};

/** The ticket paper of each shop tier: plain, silver, gold */
export const TIER_PAPER: Record<number, string> = { 1: '#ffe7b8', 2: '#e6ecf5', 3: '#ffd970' };

/** The paper a ticket is printed on: its seal's, else its tier's */
export const paperOf = (t: { tier?: number | null; seal?: Seal | null }) =>
	t.seal ? SEAL_LOOK[t.seal.theme].paper : t.tier ? TIER_PAPER[t.tier] : undefined;

/** What a seal does, in short: "×3 coins · Always wins" */
export function sealEffects(i18n: I18n, seal: Seal): string[] {
	const out: string[] = [];
	if (seal.times > 1) out.push(i18n.t('seal.times', { n: seal.times }));
	if (seal.sure) out.push(i18n.t('seal.sure'));
	if (seal.second) out.push(i18n.t('seal.second'));
	return out;
}

/** Where the motifs go over a ticket: a fixed scatter, so it does not move between renders */
export const MOTIF_SPOTS: [number, number][] = [
	[8, 10],
	[78, 6],
	[44, 22],
	[16, 46],
	[86, 40],
	[52, 62],
	[10, 82],
	[70, 86],
	[34, 94]
];
