<script>
  import { SvelteToast } from '@zerodevx/svelte-toast'
  const options = {
  }
</script>

<div class="app">
  <slot />
  <SvelteToast {options} />
</div>

<style>
    .app{
        background-color: var(--background-color);
        color: var(--text-color);
        transition: background-color 0.3s ease, color 0.3s ease;
    }

</style>
