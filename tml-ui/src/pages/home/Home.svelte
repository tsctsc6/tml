<script lang="ts">
  import { Column, Grid, Row } from "carbon-components-svelte";
  import Login from "./Login.svelte";
  import UserProfile from "./UserProfile.svelte";
  import { onMount } from "svelte";
  import apiClient from "../../lib/api";

  interface ReadUserInfoResponse {
    username: string;
    password: string;
  }

  let isLoggedIn = false;

  onMount(async () => {
    try {
      const response = await apiClient.get<ReadUserInfoResponse>(
        "/auth/read_user_info",
      );
      console.log("User info:", response);
      isLoggedIn = true;
    } catch (error: any) {
      if (error.response) {
        if (error.response.status === 401) {
          isLoggedIn = false;
        }
      }
    }
  });
</script>

<Grid>
  <Row>
    <Column><h1>Home</h1></Column>
  </Row>
  <Row>
    <Column>
      {#if isLoggedIn}<UserProfile />{:else}<Login />{/if}
    </Column>
  </Row>
</Grid>
