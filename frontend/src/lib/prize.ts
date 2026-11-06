import type { I18n } from './i18n/index.svelte.ts';
import { cosmeticKey } from './series.ts';
import type { FeedItem, Prize, Series } from './types.ts';

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
		case 'bundle':
			return p.items.map((x) => prizeText(i18n, x)).join(' + ');
	}
}

/** Coins a prize pays, bundled ones included */
export const prizeCoins = (p: Prize): number =>
	p.kind === 'coins'
		? p.amount
		: p.kind === 'bundle'
			? p.items.reduce((n, x) => n + prizeCoins(x), 0)
			: 0;

/** An achievement's name, or its id when the dictionary does not know it */
export const achievementName = (i18n: I18n, id: string) =>
	i18n.has(`achievement.${id}`) ? i18n.t(`achievement.${id}` as never) : id;

/** A line of the live feed: "ana won 1,000 coins on Lucky Three" */
export function feedText(i18n: I18n, item: FeedItem): string {
	if (item.type === 'feat')
		return i18n.t('feed.feat', { user: item.user, what: achievementName(i18n, item.id) });
	return i18n.t('feed.win', {
		user: item.user,
		what: prizeText(i18n, item.prize).replace(/^\+/, ''),
		series: i18n.t(`series.${item.series}`)
	});
}
