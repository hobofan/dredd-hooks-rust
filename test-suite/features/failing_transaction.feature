Feature: Failing a transaction

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
    # Source in src/bin/failing_transaction.rs
    When I run `dredd ./apiary.apib http://localhost:4567 --server="ruby server.rb" --language="dredd-hooks-rust" --hookfiles=../../target/debug/failing_transaction`
    Then the exit status should be 1
    And the output should contain:
      """
      Yay! Failed!
      """
