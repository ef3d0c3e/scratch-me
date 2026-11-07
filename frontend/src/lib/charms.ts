/**
 * Charms worn: which still go, and what a discount does to a price. The
 * server spends every charge; this only shows them.
 */

import type { I18n } from './i18n/index.svelte.ts';
import { countdown } from './time.ts';
import type { CharmKind, CharmTier, WornCharm } from './types.ts';

export const CHARM_ICON: Record<CharmKind, string> = {
	discount: '🏷️',
	luck: '🍀',
	payout: '💰',
	scholar: '📚',
	hourglass: '⏳',
	quest: '🧭',
	shield: '🛡️'
};
export const CHARM_KINDS: CharmKind[] = [
	'discount',
	'luck',
	'payout',
	'scholar',
	'hourglass',
	'quest',
	'shield'
];

/** Charms that only last a time: their one charge is never spent */
export const timeOnly = (kind: CharmKind) => kind === 'hourglass' || kind === 'quest';

/** Charms about tickets: they say which ones they reach */
export const onTickets = (kind: CharmKind) =>
	kind === 'discount' || kind === 'luck' || kind === 'payout' || kind === 'scholar';

/** "3 uses", "1 missed day", or nothing for a charm that only lasts a time */
export function usesText(i18n: I18n, kind: CharmKind, n: number, left = false) {
	if (timeOnly(kind)) return '';
	if (kind === 'shield') return i18n.t('charm.days_covered', { n });
	return i18n.t(left ? 'charm.left' : 'charm.uses', { n });
}

/** Time left on a charm: "1:04:59", or "6 days" past a day */
export function timeLeft(i18n: I18n, at: string, now: number) {
	const days = Math.floor((new Date(at).getTime() - now) / 86_400_000);
	return days >= 1 ? i18n.t('charm.days', { n: days }) : countdown(at, now);
}
export const CHARM_TIERS: CharmTier[] = ['bronze', 'silver', 'gold'];

/** Charms with charges left and time to go */
export const going = (charms: WornCharm[], now: number) =>
	charms.filter((c) => c.charges > 0 && new Date(c.expires_at).getTime() > now);

/** Whether a charm reaches a ticket of this price */
export const reaches = (c: { reach: number | null }, price: number) =>
	c.reach === null || price <= c.reach;

/** What a shop ticket costs with the discount charm worn, as the server counts it */
export function ticketPrice(charms: WornCharm[], price: number, now: number) {
	const d = going(charms, now).find((c) => c.kind === 'discount');
	const saved = d && reaches(d, price) ? Math.floor((price * d.strength) / 100) : 0;
	return { price: price - saved, saved };
}

/** "30 min", "1 h", "1 h 30", "7 d" */
export function duration(minutes: number) {
	if (minutes >= 1440 && minutes % 1440 === 0) return `${minutes / 1440} d`;
	const h = Math.floor(minutes / 60);
	const m = minutes % 60;
	if (h === 0) return `${m} min`;
	return m ? `${h} h ${String(m).padStart(2, '0')}` : `${h} h`;
}
