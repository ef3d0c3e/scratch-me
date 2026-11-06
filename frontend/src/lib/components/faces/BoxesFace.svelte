<script lang="ts">
	import { onMount } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { look, move } from '#lib/play.ts';
	import type { Scratched, TicketDto } from '#lib/types.ts';

	/** Pick a box; the host opens an empty one; keep or switch */
	interface Props {
		ticket: TicketDto;
		ondone: (r: Scratched) => void;
		onerror: (e: unknown) => void;
		onbeat: (good: boolean) => void;
	}
	let p: Props = $props();
	const i18n = getI18n();
	type View = {
		picked: number | null;
		opened: number | null;
		last: number | null;
		prize_box: number | null;
		coins: number | null;
		consolation: number;
	};
	let view = $state<View | null>(null);
	let busy = $state(false);

	onMount(() => {
		look<View>(p.ticket.id)
			.then((s) => (view = s.view))
			.catch(p.onerror);
	});

	async function act(mv: unknown) {
		busy = true;
		try {
			const s = await move<View>(p.ticket.id, mv);
			view = s.view;
			if (s.done) {
				p.onbeat(s.view.last === s.view.prize_box);
				p.ondone(s.done);
			}
		} catch (e) {
			p.onerror(e);
		} finally {
			busy = false;
		}
	}
	const step = $derived(
		!view ? 'wait' : view.last !== null ? 'over' : view.picked === null ? 'pick' : 'decide'
	);
</script>

<p class="rules">{i18n.t('series.mystery_boxes.rules')}</p>
<p class="say" data-testid="boxes-step">
	{#if step === 'pick'}{i18n.t('boxes.pick')}{:else if step === 'decide'}{i18n.t(
			'boxes.decide'
		)}{:else if step === 'over'}{view?.last === view?.prize_box
			? i18n.t('boxes.won')
			: i18n.t('boxes.lost')}{/if}
</p>
<div class="boxes">
	{#each { length: 3 }, b (b)}
		{@const opened = view?.opened === b}
		{@const isPrize = view?.prize_box === b}
		<button
			type="button"
			class="box"
			class:picked={view?.picked === b && step === 'decide'}
			class:final={view?.last === b}
			class:opened
			disabled={step !== 'pick' || busy}
			onclick={() => act({ kind: 'pick', pick: b })}
			aria-label={i18n.t('boxes.box', { n: b + 1 })}
			data-box={b}
		>
			<span class="lid" class:off={opened || step === 'over'}>🎁</span>
			<span class="inside">
				{#if opened}{i18n.t('boxes.empty')}{:else if step === 'over'}{isPrize
						? `💰 ${i18n.num(view?.coins ?? 0)}`
						: `🪙 ${view?.consolation}`}{/if}
			</span>
		</button>
	{/each}
</div>
{#if step === 'decide'}
	<div class="choice">
		<button type="button" class="btn ghost" disabled={busy} onclick={() => act({ kind: 'keep' })}
			>{i18n.t('boxes.keep')}</button
		>
		<button
			type="button"
			class="btn primary"
			disabled={busy}
			onclick={() => act({ kind: 'switch' })}>{i18n.t('boxes.switch')}</button
		>
	</div>
{/if}

<style>
	.say {
		text-align: center;
		font-weight: 800;
		min-height: 1.4em;
		margin-bottom: 10px;
	}
	.boxes {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 10px;
	}
	.box {
		position: relative;
		aspect-ratio: 1;
		border-radius: 16px;
		border: 0;
		background: linear-gradient(145deg, #a78bfa, #6d28d9);
		display: grid;
		place-items: center;
		box-shadow: 0 6px 0 #4c1d95;
	}
	.box.picked {
		outline: 4px solid #ffc23d;
		transform: translateY(-6px);
	}
	.box.final {
		outline: 4px solid #ffc23d;
	}
	.box.opened {
		background: #ddd6fe;
		box-shadow: none;
	}
	.lid {
		font-size: 44px;
		transition: transform 0.5s ease;
	}
	.lid.off {
		transform: translateY(-140%) rotate(-20deg);
		opacity: 0;
	}
	.inside {
		position: absolute;
		font: 400 16px/1.1 var(--display);
		color: #fff;
		text-align: center;
	}
	.opened .inside {
		color: #6d28d9;
	}
	.choice {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 10px;
		margin-top: 14px;
	}
</style>
