<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { PotTicketLayout } from '#lib/types.ts';

	/** One instant prize under the foil; the entry in the pot is already in */
	interface Props {
		layout: PotTicketLayout | null;
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

<p class="rules">{i18n.t('series.pot_ticket.rules')}</p>
<div class="play jar-play">
	<div class="instant" class:win={layout && layout.instant > 0}>
		<small>{i18n.t('pot.instant')}</small>
		{#if layout}
			<b data-testid="instant">{layout.instant > 0 ? `🪙 ${layout.instant}` : '—'}</b>
		{/if}
	</div>
	{@render foil()}
</div>
<p class="entered">🍯 {i18n.t('pot.entered')}</p>

<style>
	.jar-play {
		aspect-ratio: 2 / 1 !important;
	}
	.instant {
		height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 6px;
		background: radial-gradient(circle, #fef9c3, #fde68a);
		color: #713f12;
	}
	.instant small {
		font-size: 12px;
		font-weight: 800;
		text-transform: uppercase;
		letter-spacing: 0.08em;
	}
	.instant b {
		font: 400 40px/1 var(--display);
	}
	.instant.win b {
		color: #a16207;
		text-shadow: 0 2px 0 #fff;
	}
	.entered {
		text-align: center;
		font-size: 13px;
		font-weight: 700;
		margin-top: 10px;
	}
</style>
