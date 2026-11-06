<script lang="ts">
	import { onMount } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { look, move } from '#lib/play.ts';
	import type { Scratched, TicketDto } from '#lib/types.ts';

	/**
	 * Launch, watch the multiplier climb, cash out before the crash. The
	 * screen counts on its own clock; the server measures the cash out and
	 * says when it crashed.
	 */
	interface Props {
		ticket: TicketDto;
		ondone: (r: Scratched) => void;
		onerror: (e: unknown) => void;
		onbeat: (good: boolean) => void;
		ontick: () => void;
	}
	let p: Props = $props();
	const i18n = getI18n();
	type View = {
		launched: boolean;
		ms_flown: number | null;
		doubling: number;
		stake: number;
		cashed: number | null;
		crashed: boolean;
		crash: number | null;
	};
	let view = $state<View | null>(null);
	let busy = $state(false);
	let shown = $state(1);
	let since = 0;
	let frame = 0;
	let poll: ReturnType<typeof setInterval> | undefined;

	const flying = $derived(!!view?.launched && view.cashed === null && !view.crashed);

	function update(v: View) {
		view = v;
		if (v.ms_flown !== null) since = performance.now() - v.ms_flown;
	}

	function animate() {
		if (!view || !flying) return;
		const flown = (performance.now() - since) / 1000;
		shown = Math.floor(Math.pow(2, flown / view.doubling) * 100) / 100;
		frame = requestAnimationFrame(animate);
	}

	function finish(done: Scratched | null) {
		if (!done) return;
		clearInterval(poll);
		cancelAnimationFrame(frame);
		if (view?.cashed) shown = view.cashed;
		else if (view?.crash) shown = view.crash;
		p.onbeat(!!view?.cashed);
		p.ondone(done);
	}

	onMount(() => {
		look<View>(p.ticket.id)
			.then((s) => {
				update(s.view);
				if (flying) start();
				finish(s.done);
			})
			.catch(p.onerror);
		return () => {
			clearInterval(poll);
			cancelAnimationFrame(frame);
		};
	});

	/** While it flies: animate, and ask the server whether it crashed */
	function start() {
		frame = requestAnimationFrame(animate);
		poll = setInterval(async () => {
			p.ontick();
			try {
				const s = await look<View>(p.ticket.id);
				update(s.view);
				finish(s.done);
			} catch (e) {
				// Over already (cashed in another tab): the sheet's own error handles it
				clearInterval(poll);
				p.onerror(e);
			}
		}, 500);
	}

	async function act(kind: 'launch' | 'cash') {
		busy = true;
		try {
			const s = await move<View>(p.ticket.id, { kind });
			update(s.view);
			if (kind === 'launch' && flying) start();
			finish(s.done);
		} catch (e) {
			p.onerror(e);
		} finally {
			busy = false;
		}
	}
</script>

<p class="rules">{i18n.t('series.rocket.rules')}</p>
<div
	class="sky"
	class:flying
	class:crashed={view?.crashed}
	class:cashed={view?.cashed !== null && view?.cashed !== undefined}
>
	<span class="rocket" style:--h="{Math.min(80, Math.log2(shown) * 18)}%"
		>{view?.crashed ? '💥' : '🚀'}</span
	>
	<b class="mult" data-testid="multiplier">×{shown.toFixed(2)}</b>
	{#if view?.crashed && view.crash}
		<p class="note">{i18n.t('rocket.crashed', { m: view.crash.toFixed(2) })}</p>
	{:else if view?.cashed}
		<p class="note">{i18n.t('rocket.cashed', { m: view.cashed.toFixed(2) })}</p>
	{:else if flying}
		<p class="note">
			{i18n.t('rocket.worth', { n: i18n.num(Math.floor((view?.stake ?? 0) * shown)) })}
		</p>
	{/if}
</div>
{#if view && !view.launched}
	<button type="button" class="btn primary big" disabled={busy} onclick={() => act('launch')}
		>🚀 {i18n.t('rocket.launch')}</button
	>
{:else if flying}
	<button type="button" class="btn primary big" disabled={busy} onclick={() => act('cash')}
		>{i18n.t('rocket.cash')}</button
	>
{/if}

<style>
	.sky {
		position: relative;
		height: 240px;
		border-radius: 16px;
		overflow: hidden;
		background: linear-gradient(#0c1445, #1e3a8a 60%, #38bdf8);
		display: grid;
		place-items: center;
	}
	.sky.flying {
		background-size: 100% 200%;
		animation: climb 2s linear infinite;
	}
	@keyframes climb {
		to {
			background-position: 0 100%;
		}
	}
	.sky.crashed {
		background: linear-gradient(#450a0a, #b91c1c);
	}
	.sky.cashed {
		background: linear-gradient(#14532d, #22c55e);
	}
	.rocket {
		position: absolute;
		/* Off to the side, so the multiplier stays readable */
		left: 18%;
		bottom: calc(10% + var(--h));
		font-size: 40px;
		transform: translateX(-50%) rotate(-45deg);
		transition: bottom 0.3s linear;
	}
	.mult {
		font: 400 48px/1 var(--display);
		color: #fff;
		text-shadow: 0 3px 0 rgba(0, 0, 0, 0.3);
		z-index: 1;
	}
	.note {
		position: absolute;
		bottom: 12px;
		color: #fff;
		font-weight: 800;
	}
	.big {
		width: 100%;
		margin-top: 12px;
		min-height: 56px;
		font-size: 18px;
	}
</style>
