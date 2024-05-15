import { toast } from '@zerodevx/svelte-toast';
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
export const success = m => toast.push(m, {
	theme: {
		'--toastColor': 'mintcream',
		'--toastBackground': 'rgba(72,187,120,0.9)',
		'--toastBarBackground': '#2F855A'
	}
})
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
export const failure = m => toast.push(m, { theme: {
		'--toastColor': 'white', // текстовый цвет
		'--toastBackground': 'rgba(235, 87, 87, 0.9)', // цвет фона
		'--toastBarBackground': '#b71c1c' // цвет полосы
	} })