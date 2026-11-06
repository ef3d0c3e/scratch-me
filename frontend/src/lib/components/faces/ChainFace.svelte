<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { SERIES_LOOK } from '#lib/series.ts';
	import type { ChainLayout } from '#lib/types.ts';

	interface Props {
		layout: ChainLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
	}
	let { layout, revealed, foil, onsettled }: Props = $props();
	const i18n = getI18n();

	$effect(() => {
		if (revealed && layout) onsettled();
	});
</script>

<p class="rules">{i18n.t('series.ticket_chain.rules')}</p>
<div class="play wide">
	<div class="panels">
		{#each { length: 6 }, i (i)}
			{@const s = layout?.panels[i]}
			<div class="panel" class:found={revealed && !!s} style:--i={i}>
				{#if s}
					<span
						class="stub"
						style:--edge={SERIES_LOOK[s].edge}
						style:--edge-ink={SERIES_LOOK[s].ink}
						data-series={s}>{i18n.t(`series.${s}`)}</span
					>
				{:else if layout}
					<span class="none" aria-label={i18n.t('chain.empty')}>✕</span>
				{/if}
			</div>
		{/each}
	</div>
	{@render foil()}
</div>

<style>
	.panels {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		grid-template-rows: repeat(2, 1fr);
		gap: 8px;
		padding: 10px;
		height: 100%;
	}
	.panel {
		display: grid;
		place-items: center;
		padding: 8px;
		border-radius: 12px;
		background: var(--paper);
		border: 2px dashed #e9cf9a;
	}
	.stub {
		width: 100%;
		padding: 10px 6px;
		border-radius: 8px;
		text-align: center;
		font: 400 12px/1.1 var(--display);
		color: var(--edge-ink);
		background: var(--paper-2);
		box-shadow: 0 0 0 2.5px var(--edge);
	}
	.found {
		background: radial-gradient(circle, #f4ecff, #d9c6ff);
		border: 2px solid #7a4fe0;
		animation: lift 0.7s cubic-bezier(0.3, 1.6, 0.5, 1) both;
		animation-delay: calc(var(--i) * 60ms + 120ms);
	}
	@keyframes lift {
		40% {
			transform: translateY(-8px) rotate(-4deg);
		}
	}
	.none {
		font-size: 22px;
		opacity: 0.35;
	}
</style>
