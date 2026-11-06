<script lang="ts">
	import { onMount } from 'svelte';
	import { show, type Tier } from '#lib/fx/celebrate.ts';
	import { burst, rain, step, type Particle } from '#lib/fx/particles.ts';

	/**
	 * Confetti from the ticket, and for the bigger prizes coins and gems
	 * raining over the whole screen. Draws on its own canvas above everything
	 * and never takes a tap.
	 */
	interface Props {
		tier: Tier;
		/** Where the confetti bursts from, in viewport px */
		origin: { x: number; y: number };
	}
	let { tier, origin }: Props = $props();

	let canvas = $state<HTMLCanvasElement>();

	onMount(() => {
		if (matchMedia('(prefers-reduced-motion: reduce)').matches) return;
		const s = show(tier);
		const width = innerWidth;
		const height = innerHeight;
		const dpr = Math.min(devicePixelRatio || 1, 2);
		canvas!.width = width * dpr;
		canvas!.height = height * dpr;
		const ctx = canvas!.getContext('2d');
		if (!ctx) return;
		ctx.scale(dpr, dpr);
		let alive: Particle[] = [
			...burst(origin.x, origin.y, s.confetti),
			...rain(width, 'coin', s.coins, s.waves),
			...rain(width, 'gem', s.gems, s.waves)
		];
		let frame = 0;
		let prev = 0;
		const tick = (t: number) => {
			const dt = Math.min(32, prev ? t - prev : 16);
			prev = t;
			ctx.clearRect(0, 0, width, height);
			alive = alive.filter((p) => step(p, dt, height));
			for (const p of alive) if (p.delay <= 0) draw(ctx, p);
			frame = alive.length ? requestAnimationFrame(tick) : 0;
		};
		frame = requestAnimationFrame(tick);
		return () => cancelAnimationFrame(frame);
	});

	function draw(ctx: CanvasRenderingContext2D, p: Particle) {
		ctx.save();
		ctx.globalAlpha = Math.min(1, p.life * 3);
		ctx.translate(p.x, p.y);
		if (p.kind === 'confetti') {
			ctx.rotate(p.angle);
			// A paper strip turning over: its height swings with the spin
			ctx.scale(1, Math.cos(p.angle * 2.3));
			ctx.fillStyle = p.color;
			ctx.fillRect(-p.size / 2, -p.size / 4, p.size, p.size / 2);
		} else if (p.kind === 'coin') {
			// A coin flipping: squeezed sideways as it turns
			ctx.scale(Math.max(0.12, Math.abs(Math.cos(p.angle))), 1);
			const r = p.size / 2;
			ctx.fillStyle = '#b86e00';
			ctx.beginPath();
			ctx.arc(0, r * 0.12, r, 0, Math.PI * 2);
			ctx.fill();
			ctx.fillStyle = '#ffc23d';
			ctx.beginPath();
			ctx.arc(0, 0, r, 0, Math.PI * 2);
			ctx.fill();
			ctx.strokeStyle = '#e08e0b';
			ctx.lineWidth = r * 0.16;
			ctx.beginPath();
			ctx.arc(0, 0, r * 0.68, 0, Math.PI * 2);
			ctx.stroke();
		} else {
			ctx.rotate(Math.sin(p.angle) * 0.4);
			const r = p.size / 2;
			ctx.fillStyle = '#9fdcff';
			ctx.beginPath();
			ctx.moveTo(-r * 0.6, -r * 0.6);
			ctx.lineTo(r * 0.6, -r * 0.6);
			ctx.lineTo(r, -r * 0.1);
			ctx.lineTo(0, r);
			ctx.lineTo(-r, -r * 0.1);
			ctx.closePath();
			ctx.fill();
			ctx.fillStyle = '#57b8ff';
			ctx.beginPath();
			ctx.moveTo(-r, -r * 0.1);
			ctx.lineTo(r, -r * 0.1);
			ctx.lineTo(0, r);
			ctx.closePath();
			ctx.fill();
		}
		ctx.restore();
	}
</script>

<canvas bind:this={canvas} class="celebration" aria-hidden="true"></canvas>

<style>
	canvas {
		position: fixed;
		inset: 0;
		width: 100vw;
		height: 100dvh;
		pointer-events: none;
		z-index: 70;
	}
</style>
