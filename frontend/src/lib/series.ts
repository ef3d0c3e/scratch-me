import type { Cosmetic, Series, ThemeId } from './types.ts';

/** Each series' colours: the ticket's edge and its title */
export const SERIES_LOOK: Record<Series, { edge: string; ink: string; shadow: string }> = {
	lucky_three: { edge: '#ff4f8b', ink: '#ff4f8b', shadow: '#8a1240' },
	number_match: { edge: '#57b8ff', ink: '#1f6fc2', shadow: '#0c3a6e' },
	ticket_chain: { edge: '#b28cff', ink: '#7a4fe0', shadow: '#3d1f86' },
	cash_wheel: { edge: '#34e0a1', ink: '#119469', shadow: '#064d36' },
	golden: { edge: '#ffc23d', ink: '#c77d00', shadow: '#6b3d00' },
	high_card: { edge: '#ff6b6f', ink: '#c0262c', shadow: '#6e0f12' },
	lucky_reels: { edge: '#ff9f1c', ink: '#c15f00', shadow: '#5e2e00' },
	bingo: { edge: '#a3e635', ink: '#4d7c0f', shadow: '#24400a' },
	treasure_dig: { edge: '#c08a4a', ink: '#7a4a12', shadow: '#3d2408' },
	tic_tac_toe: { edge: '#38bdf8', ink: '#0369a1', shadow: '#082f49' },
	lucky_dice: { edge: '#f43f5e', ink: '#be123c', shadow: '#4c0519' },
	derby: { edge: '#16a34a', ink: '#166534', shadow: '#052e16' },
	plinko: { edge: '#d946ef', ink: '#a21caf', shadow: '#4a044e' },
	push_luck: { edge: '#f97316', ink: '#c2410c', shadow: '#431407' },
	safe_cracker: { edge: '#64748b', ink: '#334155', shadow: '#0f172a' },
	mystery_boxes: { edge: '#8b5cf6', ink: '#6d28d9', shadow: '#2e1065' },
	defuse_bomb: { edge: '#ef4444', ink: '#b91c1c', shadow: '#450a0a' },
	rocket: { edge: '#0ea5e9', ink: '#0369a1', shadow: '#082f49' },
	blackjack: { edge: '#15803d', ink: '#14532d', shadow: '#052e16' },
	hot_spot: { edge: '#f43f5e', ink: '#be123c', shadow: '#4c0519' },
	battleship: { edge: '#1d4ed8', ink: '#1e3a8a', shadow: '#0b1640' },
	board_walk: { edge: '#eab308', ink: '#a16207', shadow: '#422006' },
	fishing: { edge: '#06b6d4', ink: '#0e7490', shadow: '#083344' },
	word_bank: { edge: '#84cc16', ink: '#4d7c0f', shadow: '#1a2e05' },
	word_search: { edge: '#ec4899', ink: '#be185d', shadow: '#500724' },
	pumpkin_patch: { edge: '#f97316', ink: '#9a3412', shadow: '#431407' },
	snowball: { edge: '#7dd3fc', ink: '#0369a1', shadow: '#082f49' },
	pot_ticket: { edge: '#facc15', ink: '#a16207', shadow: '#422006' },
	duel: { edge: '#e11d48', ink: '#9f1239', shadow: '#4c0519' },
	coin_flip: { edge: '#f59e0b', ink: '#b45309', shadow: '#451a03' },
	roulette: { edge: '#dc2626', ink: '#991b1b', shadow: '#1b1530' },
	poker_hand: { edge: '#059669', ink: '#065f46', shadow: '#022c22' },
	mega_reels: { edge: '#f97316', ink: '#9a3412', shadow: '#3b1300' },
	platinum_three: { edge: '#a5b4fc', ink: '#4338ca', shadow: '#1e1b4b' }
};

/** How the foil over each series splits into squares for the reveal rule */
export const FOIL_GRID: Record<Series, { cols: number; rows: number }> = {
	lucky_three: { cols: 3, rows: 3 },
	number_match: { cols: 4, rows: 3 },
	ticket_chain: { cols: 3, rows: 2 },
	cash_wheel: { cols: 1, rows: 1 },
	golden: { cols: 2, rows: 2 },
	high_card: { cols: 4, rows: 2 },
	lucky_reels: { cols: 3, rows: 1 },
	bingo: { cols: 5, rows: 6 },
	/** Each spot of the beach has its own little foil */
	treasure_dig: { cols: 1, rows: 1 },
	tic_tac_toe: { cols: 3, rows: 1 },
	lucky_dice: { cols: 3, rows: 1 },
	derby: { cols: 2, rows: 1 },
	plinko: { cols: 1, rows: 1 },
	push_luck: { cols: 1, rows: 1 },
	safe_cracker: { cols: 1, rows: 1 },
	mystery_boxes: { cols: 1, rows: 1 },
	defuse_bomb: { cols: 1, rows: 1 },
	rocket: { cols: 1, rows: 1 },
	blackjack: { cols: 1, rows: 4 },
	hot_spot: { cols: 5, rows: 4 },
	/** Each cell of the sea has its own little foil */
	battleship: { cols: 1, rows: 1 },
	board_walk: { cols: 3, rows: 1 },
	fishing: { cols: 4, rows: 1 },
	word_bank: { cols: 6, rows: 2 },
	word_search: { cols: 4, rows: 4 },
	pumpkin_patch: { cols: 3, rows: 3 },
	snowball: { cols: 5, rows: 1 },
	pot_ticket: { cols: 1, rows: 1 },
	/** Both rows of dice */
	duel: { cols: 3, rows: 2 },
	/** Each cell has its own foil: scratching one is the call */
	coin_flip: { cols: 1, rows: 1 },
	roulette: { cols: 1, rows: 1 },
	poker_hand: { cols: 5, rows: 1 },
	mega_reels: { cols: 3, rows: 1 },
	platinum_three: { cols: 3, rows: 3 }
};

/** Series played move by move, not scratched all at once */
export const PLAYED: Series[] = [
	'treasure_dig',
	'push_luck',
	'safe_cracker',
	'mystery_boxes',
	'defuse_bomb',
	'rocket',
	'battleship',
	'coin_flip',
	'roulette'
];
export const isPlayed = (s: Series) => PLAYED.includes(s);

/** A theme's colours at a glance: background, panel, accent, foil */
export const SKIN_SWATCH: Record<ThemeId, [string, string, string, string]> = {
	midnight: ['#140b2e', '#2b1a5c', '#ffc23d', '#a9a3c4'],
	sunset: ['#2a0f1f', '#5a1f42', '#ffb03d', '#c98f6f'],
	mint: ['#06231e', '#11463d', '#ffd166', '#9fc9b9'],
	neon: ['#05010f', '#1d0c3d', '#f7ff3c', '#6a4fd0'],
	royal: ['#0b0e2a', '#1e2766', '#ffd479', '#c9a24a'],
	ocean: ['#03203a', '#0b4a73', '#7dd3fc', '#9cc7d9'],
	candy: ['#3a0d2e', '#7a1f5c', '#ffb3d9', '#f5d0e6'],
	arcade: ['#0a0a12', '#1f1f3a', '#00f0ff', '#ff3cac'],
	forest: ['#0d1f12', '#1f3d24', '#d9b26a', '#7f9a6b'],
	champion: ['#1a0f00', '#3d2400', '#ffd700', '#e6c36a'],
	spooky: ['#0f0a14', '#2a1438', '#f97316', '#6b5b7b'],
	frost: ['#0b1d2e', '#16324d', '#bae6fd', '#e0f2fe']
};

/** The dictionary key naming a cosmetic */
export const cosmeticKey = (item: Cosmetic) =>
	item.kind === 'theme'
		? (`theme.${item.id}` as const)
		: item.kind === 'tool'
			? (`tool.${item.id}` as const)
			: (`foil.${item.id}` as const);
