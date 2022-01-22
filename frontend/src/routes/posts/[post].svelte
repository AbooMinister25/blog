<script lang="ts" context="module">
    import type { Load } from "@sveltejs/kit";
    import { variables } from "$lib/variables";
    import Nav from "$lib/Nav.svelte";

    export const load: Load = async ({ page: { params }, fetch }) => {
        const { slug } = params;

        const res = await fetch(variables.apiUrl + "/posts/" + params['post']);

        if (res.status != 200) {
            const data = await res.json();
            const message = data["message"];

            const error = new Error(message);
            return { status: res.status, error };
        } else {
            const data = await res.json();
            return { props: { post: data["data"] } };
        }
    };
</script>

<script lang="ts">
    import type { Post } from "$lib/types";

    export let post: Post;
</script>

<div class='post'>
    <Nav />
    <div>
        {@html post.body}
    </div>
</div>
