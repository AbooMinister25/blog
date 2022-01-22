<script lang="ts" context="module">
    import type { ErrorLoad } from "@sveltejs/kit";
    export type { ErrorLoad } from "@sveltejs/kit";

    export const load: ErrorLoad = ({ error, status }) => {
        return { props: { error, status } };
    };
</script>

<script lang="ts">
    export let error: Error;
    export let status: number;
</script>

<div class="error-page">
    {#if status === 404}
        <h1 class="error-header">Page not found D:</h1>
        <p class="error-desc">
            That url doesn't seem to exist on this site, do you want to head <span
                class="home-link"><a href="/">home?</a></span
            >
        </p>
    {:else}
        <h1 class="error-header">Error {status}</h1>
        <p class="error-desc">{error.message}</p>
    {/if}
</div>
