<script lang="ts">
  import { Button, Column, Grid, Row } from "carbon-components-svelte";

  async function handleClick(e: Event) {
    if (typeof window === "undefined") {
      return;
    }
    localStorage.removeItem("token");
    window.location.reload();
  }
</script>

<Grid>
  <Row>
    <Column>
      <h2>User Profile</h2>
    </Column>
  </Row>
  <Row
    ><Column>
      <Button on:click={handleClick}>Logout</Button>
    </Column></Row
  >
</Grid>
