<script lang="ts">
	//import type { PageData } from './$houdini';
	import { ProductionInfoSelectByDateStore } from '$houdini';
	import type {
		ProductionInfoSelectByDate$result,
		ProductionInfoSelectByDate$input
	} from '$houdini';

	//$: ({ ProductionInfoSelectByDate } = data);
	let store = new ProductionInfoSelectByDateStore();

	let selectedDate: string;
	let isoDate = '';
	let loading: boolean;

	function convertToISO(date: string) {
		const d = new Date(date);
		// Set time to 00:00:00
		d.setUTCHours(0, 0, 0, 0);
		return d.toISOString();
	}

	async function handleDateChange(event: Event) {
		const target = event.target as HTMLInputElement;
		selectedDate = target.value;

		isoDate = convertToISO(selectedDate);

		// Fetch the data with the selected date
		loading = true;
		await store.fetch({ variables: { date: isoDate } });
		//const result = await query({ date: isoDate });

		//data = result.data;
		loading = false;
	}

	function getDateFromISO(isoString: string): string {
		const date = new Date(isoString);

		// Ensure date is valid
		if (isNaN(date.getTime())) {
			throw new Error('Invalid ISO Date String');
		}

		const year = date.getFullYear();

		const month = String(date.getMonth() + 1).padStart(2, '0'); // Months are zero-indexed
		const day = String(date.getDate()).padStart(2, '0');

		return `${year}-${month}-${day}`;
	}
</script>

<div>
	<div>Dashboard!</div>
	<br />

	<input type="date" bind:value={selectedDate} on:change={handleDateChange} />
	<p>Выберите дату: {isoDate}</p>

	{#if $store}
		{#if $store.fetching}
			<p>Загрузка...</p>
		{:else if $store.data?.productionInfo.selectByDate}
			<div>Дата: {getDateFromISO($store.data?.productionInfo.selectByDate?.date)}</div>
			<div>
				Реально произведено: {$store.data?.productionInfo.selectByDate?.productionFact}
				{$store.data?.productionInfo.selectByDate.measureUnits.name}
			</div>
			<div>
				План производства за дату: {$store.data?.productionInfo.selectByDate?.productionPlan
					.amount}
				{$store.data?.productionInfo.selectByDate.measureUnits.name}
			</div>
			<div>
				План продаж за дату: {$store.data?.productionInfo.selectByDate?.salesPlan.amount}
				{$store.data?.productionInfo.selectByDate.measureUnits.name}
			</div>
      <div>
        Выходная труба/конвейр: {$store.data?.productionInfo.selectByDate?.finalPipe.name}
      </div>
		{:else}
			Данных на выбранную дату нет
		{/if}
	{/if}
</div>

