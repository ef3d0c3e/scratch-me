import type { I18n } from './i18n/index.svelte.ts';
import { cosmeticKey } from './series.ts';
import type { Prize, Series } from './types.ts';

/** "2 × Lucky Three + Golden Ticket" */
export function ticketsText(i18n: I18n, series: Series[]) {
	const counts = new Map<Series, number>();
	for (const s of series) counts.set(s, (counts.get(s) ?? 0) + 1);
	return [...counts]
		.map(([s, n]) => (n > 1 ? `${n} × ${i18n.t(`series.${s}`)}` : i18n.t(`series.${s}`)))
		.join(' + ');
}

/** A reward in a few words: "+100 coins", "Number Match", "Neon" */
export function prizeText(i18n: I18n, p: Prize): string {
	switch (p.kind) {
		case 'nothing':
			return '';
		case 'coins':
			return i18n.t('lobby.got_coins', { n: p.amount });
		case 'ticket':
			return i18n.t(`series.${p.series}`);
		case 'tickets':
			return ticketsText(i18n, p.series);
		case 'pocket_refill':
			return i18n.t('scratch.prize.pocket_refill');
		case 'cosmetic':
			return i18n.t(cosmeticKey(p.item));
	}
}
