<script lang="ts">
	import { onMount } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import { look, move } from '#lib/play.ts';
	import type { Scratched, TicketDto } from '#lib/types.ts';

	/** Six wires and a fuse that burns on the server; the screen counts down with it */
	interface Props {
		ticket: TicketDto;
		ondone: (r: Scratched) => void;
		onerror: (e: unknown) => void;
		onbeat: (good: boolean) => void;
		ontick: () => void;
	}
	let p: Props = $props();
	const i18n = getI18n();
	type Wire = 'defuse' | 'boom' | 'dud';
	type View = {
		coins: number;
		cut: { wire: number; is: Wire }[];
		ms_left: number;
		late: boolean;
		wires: Wire[] | null;
	};
	const COLORS = ['#ef4444', '#3b82f6', '#22c55e', '#facc15', '#f8fafc', '#a855f7'];

	let view = $state<View | null>(null);
	let busy = $state(false);
	/** When the fuse runs out, by this screen's clock */
	let ends = 0;
	let left = $state(20_000);
	let timer: ReturnType<typeof setInterval> | undefined;
	const over = $derived(!!view?.wires);
	const boom = $derived(!!view && (view.late || view.cut.some((c) => c.is === 'boom')));
	const defused = $derived(!!view?.cut.some((c) => c.is === 'defuse'));

	function update(v: View) {
		view = v;
		ends = performance.now() + v.ms_left;
	}

	onMount(() => {
		look<View>(p.ticket.id)
			.then((s) => {
				update(s.view);
				if (s.done) p.ondone(s.done);
			})
			.catch(p.onerror);
		let lastSecond = -1;
		timer = setInterval(async () => {
			if (!view || over) return;
			left = Math.max(0, ends - performance.now());
			const sec = Math.ceil(left / 1000);
			if (sec !== lastSecond) {
				lastSecond = sec;
				p.ontick();
			}
			// Out of time here: the server has the last word
			if (left === 0 && !busy) {
				busy = true;
				try {
					const s = await look<View>(p.ticket.id);
					update(s.view);
					if (s.done) {
						p.onbeat(false);
						p.ondone(s.done);
					}
				} catch (e) {
					p.onerror(e);
				} finally {
					busy = false;
				}
			}
		}, 100);
		return () => clearInterval(timer);
	});

	async function cut(w: number) {
		if (busy || over) return;
		busy = true;
		try {
			const s = await move<View>(p.ticket.id, { cut: w });
			update(s.view);
			const last = s.view.cut.at(-1)?.is;
			if (last !== 'dud') p.onbeat(last === 'defuse');
			if (s.done) p.ondone(s.done);
		} catch (e) {
			p.onerror(e);
		} finally {
			busy = false;
		}
	}
	const isCut = (w: number) => !!view?.cut.some((c) => c.wire === w);
</script>

<p class="rules">{i18n.t('series.defuse_bomb.rules')}</p>
<div class="bomb" class:boom class:defused>
	<div class="display">
		<span class="clock" data-testid="clock" class:low={left < 5000}>
			{over ? (defused ? i18n.t('bomb.defused') : '00.0') : (left / 1000).toFixed(1)}
		</span>
		<span class="stake">💰 {view ? i18n.num(view.coins) : ''}</span>
	</div>
	<div class="wires">
		{#each COLORS as color, w (w)}
			<button
				type="button"
				class="wire"
				class:cut={isCut(w)}
				style:--c={color}
				disabled={!view || over || busy || isCut(w)}
				onclick={() => cut(w)}
				aria-label={i18n.t('bomb.cut', { n: w + 1 })}
				data-wire={w}
				data-is={view?.wires?.[w] ?? view?.cut.find((c) => c.wire === w)?.is ?? ''}
			>
				<span class="strand"></span>
				{#if over && view?.wires}<small>{i18n.t(`bomb.${view.wires[w]}`)}</small>{/if}
			</button>
		{/each}
	</div>
	{#if boom}<div class="flash" aria-hidden="true">💥</div>{/if}
</div>
{#if view?.late}<p class="late">{i18n.t('bomb.late')}</p>{/if}

<style>
	.bomb {
		position: relative;
		padding: 14px;
		border-radius: 18px;
		background: linear-gradient(#3f3f46, #18181b);
		overflow: hidden;
	}
	.bomb.boom {
		animation: shake 0.5s both;
	}
	.bomb.defused {
		background: linear-gradient(#166534, #052e16);
	}
	@keyframes shake {
		20%,
		60% {
			transform: translateX(-8px);
		}
		40%,
		80% {
			transform: translateX(8px);
		}
	}
	.display {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 10px 14px;
		border-radius: 10px;
		background: #000;
		margin-bottom: 14px;
	}
	.clock {
		font:
			700 32px/1 ui-monospace,
			monospace;
		color: #4ade80;
	}
	.clock.low {
		color: #f87171;
		animation: blink 0.5s steps(2) infinite;
	}
	@keyframes blink {
		50% {
			opacity: 0.4;
		}
	}
	.stake {
		font: 400 20px/1 var(--display);
		color: #fde68a;
	}
	.wires {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.wire {
		position: relative;
		height: 34px;
		border: 0;
		background: none;
		padding: 0 6px;
		display: flex;
		align-items: center;
	}
	.strand {
		flex: 1;
		height: 10px;
		border-radius: 5px;
		background: var(--c);
		box-shadow: inset 0 -3px 0 rgba(0, 0, 0, 0.25);
	}
	.wire.cut .strand {
		background: linear-gradient(90deg, var(--c) 45%, transparent 45% 55%, var(--c) 55%);
	}
	.wire small {
		position: absolute;
		right: 10px;
		font-weight: 800;
		color: #fff;
		text-shadow: 0 1px 2px #000;
	}
	.flash {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
		font-size: 120px;
		background: rgba(255, 120, 0, 0.5);
		animation: fade 1s ease-out both;
	}
	@keyframes fade {
		to {
			opacity: 0;
		}
	}
	.late {
		text-align: center;
		font-weight: 800;
		color: #b91c1c;
		margin-top: 8px;
	}
</style>
