<script lang="ts">
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { api, post } from '#lib/api.ts';
	import { countdown } from '#lib/time.ts';
	import { getLive } from '#lib/live.svelte.ts';
	import { getSession } from '#lib/session.svelte.ts';
	import type { Bought, PotDto } from '#lib/types.ts';

	/** Today's shared pot: how much, how many entries, and one more for 50 */
	let { now }: { now: number } = $props();
	const i18n = getI18n();
	const live = getLive();
	const session = getSession();
	const PRICE = 50;
	let pot = $state<PotDto | null>(null);
	let busy = $state(false);
	let error = $state('');

	async function load() {
		pot = await api<PotDto>('/pot').catch(() => pot);
	}
	// On show, on pot news, and past the draw
	$effect(() => {
		void live.social;
		void load();
	});
	$effect(() => {
		if (pot && Date.parse(pot.draws_at) <= now) void load();
	});

	async function enter() {
		busy = true;
		error = '';
		try {
			const r = await post<Bought>('/shop/buy', { kind: 'ticket', series: 'pot_ticket' });
			session.apply(r.progress);
			if (r.ticket) session.add([r.ticket]);
			await load();
		} catch (err) {
			error = i18n.error(err);
		} finally {
			busy = false;
		}
	}
</script>

{#if pot}
	<section class="pot" aria-labelledby="pot-title">
		<span class="jar" aria-hidden="true">🍯</span>
		<div class="what">
			<h2 id="pot-title">{i18n.t('pot.title')}</h2>
			<p class="amount num" data-testid="pot-amount">🪙 {i18n.num(pot.amount)}</p>
			<p class="small">
				{i18n.t('pot.entries', { n: pot.entries })} ·
				{pot.mine ? i18n.t('pot.mine', { n: pot.mine }) : i18n.t('pot.mine_none')}
			</p>
			<p class="small timer num">{i18n.t('pot.draws', { time: countdown(pot.draws_at, now) })}</p>
			{#if pot.last}
				<p class="small last">
					{pot.last.winner
						? i18n.t('pot.last', { name: pot.last.winner, n: i18n.num(pot.last.amount) })
						: i18n.t('pot.last_none')}
				</p>
			{/if}
			{#if error}<p class="err" role="alert">{error}</p>{/if}
		</div>
		<button
			type="button"
			class="btn small primary"
			disabled={busy || session.me.coins < PRICE}
			onclick={enter}>{i18n.t('pot.buy', { n: PRICE })}</button
		>
	</section>
{/if}

<style>
	.pot {
		display: grid;
		grid-template-columns: auto 1fr auto;
		align-items: center;
		gap: 12px;
		padding: 14px;
		border-radius: 18px;
		background: linear-gradient(135deg, rgba(250, 204, 21, 0.18), var(--night-2));
		border: 1px solid rgba(250, 204, 21, 0.45);
	}
	.jar {
		font-size: 38px;
	}
	h2 {
		font-size: 15px;
	}
	.amount {
		font: 400 26px/1.1 var(--display);
		color: #facc15;
	}
	.small {
		font-size: 12px;
		opacity: 0.85;
	}
	.err {
		font-size: 12px;
	}
</style>
