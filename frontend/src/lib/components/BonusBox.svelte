<script lang="ts">
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { Bonus, FoilId, ToolId } from '#lib/types.ts';
	import ScratchFoil from './ScratchFoil.svelte';

	/**
	 * The bonus box: its own little foil, scratched on its own or falling off
	 * with the rest of the ticket. Its effect is already in the prize.
	 */
	interface Props {
		bonus: Bonus;
		revealed: boolean;
		tool: ToolId;
		foil: FoilId;
	}
	let { bonus, revealed, tool, foil }: Props = $props();
	const i18n = getI18n();
	let mine = $state(false);
	const open = $derived(revealed || mine);
</script>

<div class="bonus" class:open data-bonus={bonus.kind}>
	<span class="label">{i18n.t('bonus.title')}</span>
	<div class="box">
		<b>
			{bonus.kind === 'times'
				? `×${bonus.times}`
				: i18n.t('bonus.ticket', { name: i18n.t(`series.${bonus.series}`) })}
		</b>
		<ScratchFoil
			revealed={open}
			onreveal={() => (mine = true)}
			label="★"
			{tool}
			{foil}
			shiny={false}
		/>
	</div>
</div>

<style>
	.bonus {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 10px;
		margin-top: 12px;
	}
	.label {
		font: 400 14px/1 var(--display);
		color: #c77d00;
	}
	.box {
		position: relative;
		min-width: 120px;
		height: 44px;
		border-radius: 10px;
		display: grid;
		place-items: center;
		padding: 0 12px;
		background: radial-gradient(circle, #fff7d1, #ffd66b);
		border: 2px dashed #c77d00;
	}
	.box b {
		font: 400 20px/1 var(--display);
		color: #9a5a00;
		white-space: nowrap;
	}
	.open .box b {
		animation: stamp 0.6s cubic-bezier(0.3, 1.8, 0.5, 1) both;
	}
	@keyframes stamp {
		from {
			transform: scale(2.2) rotate(-10deg);
			opacity: 0;
		}
	}
</style>
