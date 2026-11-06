/**
 * Charms worn: which still go, and what a discount does to a price. The
 * server spends every charge; this only shows them.
 */

import type { CharmKind, CharmTier, WornCharm } from './types.ts';

export const CHARM_ICON: Record<CharmKind, string> = { discount: '🏷️', luck: '🍀', payout: '💰' };
export const CHARM_KINDS: CharmKind[] = ['discount', 'luck', 'payout'];
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

/** "30 min", "1 h", "1 h 30" */
export function duration(minutes: number) {
	const h = Math.floor(minutes / 60);
	const m = minutes % 60;
	if (h === 0) return `${m} min`;
	return m ? `${h} h ${String(m).padStart(2, '0')}` : `${h} h`;
}
