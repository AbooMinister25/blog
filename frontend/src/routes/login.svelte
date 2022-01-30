<script lang="ts">
    import { variables } from "$lib/variables";
    import { goto } from "$app/navigation";
    import { user } from "$lib/stores";
    import type { User } from "$lib/types";

    let message = "";
    let username = "";
    let password = "";

    async function handleSubmit() {
        const res = await fetch(variables.apiUrl + "/users/login", {
            headers: {
                username: username,
                password: password,
            },
            mode: "cors",
        });

        if (res.ok) {
            const data: User = { username: username, password: password };
            user.set(data);
            await goto("/dashboard");
        } else {
            const data = await res.json();
            if (data?.message) {
                message = data.message;
            }
        }
    }
</script>

<div>
    <form on:submit|preventDefault={handleSubmit} class="login-form">
        <div class="form-fields">
            <input
                type="text"
                id="username"
                class="form-field"
                placeholder="Username"
                bind:value={username}
            />
            <input
                type="password"
                id="password"
                class="form-field"
                placeholder="Password"
                bind:value={password}
            />
        </div>
        <button class="submit-button" type="submit">Submit</button>
    </form>
    {#if message}
        <div class="login-error">
            <p>{message}</p>
        </div>
    {/if}
</div>
