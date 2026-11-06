<script lang="ts">
	import { onMount } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { look, move } from '#lib/play.ts';
	import type { Scratched, TicketDto } from '#lib/types.ts';

	/** Three dials, five tries; each try answers higher, lower or right for every digit */
	interface Props {
		ticket: TicketDto;
		ondone: (r: Scratched) => void;
		onerror: (e: unknown) => void;
		onbeat: (good: boolean) => void;
		ontick: () => void;
	}
	let p: Props = $props();
	const i18n = getI18n();
	type Hint = 'right' | 'higher' | 'lower';
	type View = {
		tries: { guess: number[]; hints: Hint[] }[];
		left: number;
		pays: number[];
		opened: boolean;
		code: number[] | null;
	};

	let view = $state<View | null>(null);
	let dials = $state([0, 0, 0]);
	let busy = $state(false);
	const ARROW: Record<Hint, string> = { right: '✓', higher: '▲', lower: '▼' };

	onMount(() => {
		look<View>(p.ticket.id)
			.then((s) => (view = s.view))
			.catch(p.onerror);
	});

	function turn(i: number, by: number) {
		dials[i] = (dials[i] + by + 10) % 10;
		p.ontick();
	}

	async function tryCode() {
		busy = true;
		try {
			const s = await move<View>(p.ticket.id, { guess: dials });
			view = s.view;
			p.onbeat(s.view.opened);
			if (s.done) p.ondone(s.done);
		} catch (e) {
			p.onerror(e);
		} finally {
			busy = false;
		}
	}
	const tryNo = $derived(view ? view.tries.length : 0);
</script>

<p class="rules">{i18n.t('series.safe_cracker.rules')}</p>
<div class="safe" class:open={view?.opened}>
	<div class="dials">
		{#each dials as d, i (i)}
			<div class="dial">
				<button
					type="button"
					onclick={() => turn(i, 1)}
					aria-label={i18n.t('safe.up', { n: i + 1 })}
					disabled={!!view?.code}>▲</button
				>
				<b data-testid="dial-{i}">{view?.code ? view.code[i] : d}</b>
				<button
					type="button"
					onclick={() => turn(i, -1)}
					aria-label={i18n.t('safe.down', { n: i + 1 })}
					disabled={!!view?.code}>▼</button
				>
			</div>
		{/each}
	</div>
	{#if view && !view.code}
		<button type="button" class="btn primary" disabled={busy} onclick={tryCode}
			>{i18n.t('safe.try', { n: view.left })}</button
		>
	{:else if view?.opened}
		<p class="state">🔓 {i18n.t('safe.opened')}</p>
	{:else if view?.code}
		<p class="state">🔒 {i18n.t('safe.stayed_shut')}</p>
	{/if}
</div>
<ol class="tries">
	{#each view?.pays ?? [] as pays, i (i)}
		{@const t = view?.tries[i]}
		<li class:now={i === tryNo && !view?.code} data-try={i}>
			<span class="no">{i + 1}</span>
			{#if t}
				<span class="guess">
					{#each t.guess as g, k (k)}<span class="g {t.hints[k]}"
							>{g}<i>{ARROW[t.hints[k]]}</i></span
						>{/each}
				</span>
			{:else}
				<span class="guess empty">· · ·</span>
			{/if}
			<span class="pays">{i18n.num(pays)}</span>
		</li>
	{/each}
</ol>

<style>
	.safe {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 16px;
		border-radius: 16px;
		background: linear-gradient(145deg, #64748b, #334155);
		box-shadow: inset 0 0 0 4px #1e293b;
	}
	.safe.open {
		background: linear-gradient(145deg, #fde68a, #d97706);
	}
	.dials {
		display: flex;
		gap: 12px;
	}
	.dial {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
	}
	.dial b {
		width: 56px;
		height: 64px;
		display: grid;
		place-items: center;
		border-radius: 10px;
		background: #0f172a;
		color: #fde68a;
		font: 400 36px/1 var(--display);
	}
	.dial button {
		width: 56px;
		height: 32px;
		border: 0;
		border-radius: 8px;
		background: #e2e8f0;
		color: #1e293b;
	}
	.state {
		font: 400 18px/1 var(--display);
		color: #fff;
	}
	.tries {
		list-style: none;
		margin: 12px 0 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
		font-size: 14px;
	}
	.tries li {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 4px 10px;
		border-radius: 8px;
	}
	.tries li.now {
		background: #e2e8f0;
	}
	.no {
		font-weight: 800;
		width: 16px;
	}
	.guess {
		flex: 1;
		display: flex;
		gap: 8px;
		font: 400 18px/1 var(--display);
	}
	.guess.empty {
		opacity: 0.4;
	}
	.g i {
		font-style: normal;
		font-size: 12px;
		margin-left: 2px;
	}
	.g.right {
		color: #15803d;
	}
	.g.higher,
	.g.lower {
		color: #b91c1c;
	}
	.pays {
		font-weight: 800;
		color: #334155;
	}
</style>
