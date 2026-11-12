import adapter from '@sveltejs/adapter-static';

const config = {
  kit: {
    adapter: adapter(),
    alias: {
      $components: './src/lib/components'
    }
  }
};

export default config;
