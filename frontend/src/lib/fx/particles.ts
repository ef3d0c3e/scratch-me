/** Confetti, coins and gems flying about: spawning and physics, no drawing */

export type Kind = 'confetti' | 'coin' | 'gem';

export interface Particle {
	kind: Kind;
	x: number;
	y: number;
	vx: number;
	vy: number;
	angle: number;
	spin: number;
	size: number;
	color: string;
	/** ms before it starts moving and shows */
	delay: number;
	/** 1 → 0 */
	life: number;
	/** ms of life it has in total */
	ttl: number;
}

export const CONFETTI_COLORS = ['#ffc23d', '#ff4f8b', '#34e0a1', '#57b8ff', '#b28cff', '#fff6e5'];

/** px / ms² */
export const GRAVITY = 0.0012;
/** Air slows paper down more than coins */
const DRAG: Record<Kind, number> = { confetti: 0.0025, coin: 0.0004, gem: 0.0004 };

type Random = () => number;

/** Paper bits shot upward from (x, y), fanning out */
export function burst(x: number, y: number, n: number, rnd: Random = Math.random): Particle[] {
	return Array.from({ length: n }, () => {
		const a = -Math.PI / 2 + (rnd() - 0.5) * Math.PI * 0.9;
		const speed = 0.5 + rnd() * 0.9;
		return {
			kind: 'confetti' as const,
			x,
			y,
			vx: Math.cos(a) * speed,
			vy: Math.sin(a) * speed,
			angle: rnd() * Math.PI,
			spin: (rnd() - 0.5) * 0.03,
			size: 5 + rnd() * 6,
			color: CONFETTI_COLORS[Math.floor(rnd() * CONFETTI_COLORS.length)],
			delay: rnd() * 80,
			life: 1,
			ttl: 1800 + rnd() * 900
		};
	});
}

/** Coins (and gems) falling from above the screen, across its width, in waves */
export function rain(
	width: number,
	kind: 'coin' | 'gem',
	n: number,
	waves: number,
	rnd: Random = Math.random
): Particle[] {
	return Array.from({ length: n }, (_, i) => ({
		kind,
		x: rnd() * width,
		y: -30 - rnd() * 120,
		vx: (rnd() - 0.5) * 0.15,
		vy: 0.15 + rnd() * 0.3,
		angle: rnd() * Math.PI * 2,
		spin: 0.004 + rnd() * 0.01,
		size: kind === 'coin' ? 14 + rnd() * 8 : 12 + rnd() * 6,
		color: kind === 'coin' ? '#ffc23d' : '#57b8ff',
		delay: (i % Math.max(1, waves)) * 450 + rnd() * 300,
		life: 1,
		ttl: 3200
	}));
}

/** Moves a particle on by `dt` ms; false once it is gone */
export function step(p: Particle, dt: number, height: number): boolean {
	if (p.delay > 0) {
		p.delay -= dt;
		return true;
	}
	const drag = DRAG[p.kind];
	p.vx -= p.vx * drag * dt;
	p.vy += GRAVITY * dt - p.vy * drag * dt;
	p.x += p.vx * dt;
	p.y += p.vy * dt;
	p.angle += p.spin * dt;
	p.life -= dt / p.ttl;
	return p.life > 0 && p.y < height + 40;
}
