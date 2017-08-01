Feature: Multiple hook files with a glob

  Background:
    Given I have "dredd-hooks-rust" command installed
    And I have "dredd" command installed
    And a file named "server.rb" with:
      """
      require 'sinatra'
      get '/message' do
        "Hello World!\n\n"
      end
      """

    And a file named "apiary.apib" with:
      """
      # My Api
      ## GET /message
      + Response 200 (text/html;charset=utf-8)
          Hello World!
      """

  Scenario:
    # See sources at src/bin/multiple_hookfiles*
    When I run `dredd ./apiary.apib http://localhost:4567 --server="ruby server.rb" --language="dredd-hooks-rust" --hookfiles=../../target/debug/multiple_hookfiles_hookfile1 --hookfiles=../../target/debug/multiple_hookfiles_hookfile2 --hookfiles=../../target/debug/multiple_hookfiles_hookfile_*_globed`
    Then the exit status should be 0
    And the output should contain:
      """
      It's me, File1
      """
    And the output should contain:
      """
      It's me, File2
      """
    And the output should contain:
      """
      It's me, File3
      """
